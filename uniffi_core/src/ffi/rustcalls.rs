/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Low-level support for calling rust functions
//!
//! This module helps the scaffolding code make calls to rust functions and pass back the result to the FFI bindings code.
//!
//! It handles:
//!    - Catching panics
//!    - Adapting the result of `Return::lower_return()` into either a return value or an
//!      exception

use crate::{FfiDefault, Lower, RustBuffer, UniFfiTag};
use std::mem::ManuallyDrop;
use std::panic;

/// Represents the success/error of a rust call
///
/// ## Usage
///
/// - The consumer code creates a [RustCallStatus] with an empty [RustBuffer] and
///   [RustCallStatusCode::Success] (0) as the status code
/// - A pointer to this object is passed to the rust FFI function.  This is an
///   "out parameter" which will be updated with any error that occurred during the function's
///   execution.
/// - After the call, if `code` is [RustCallStatusCode::Error] or [RustCallStatusCode::UnexpectedError]
///   then `error_buf` will be updated to contain a serialized error object.   See
///   [RustCallStatusCode] for what gets serialized. The consumer is responsible for freeing `error_buf`.
///
/// ## Layout/fields
///
/// The layout of this struct is important since consumers on the other side of the FFI need to
/// construct it.  If this were a C struct, it would look like:
///
/// ```c,no_run
/// struct RustCallStatus {
///     int8_t code;
///     RustBuffer error_buf;
/// };
/// ```
#[repr(C)]
pub struct RustCallStatus {
    pub code: RustCallStatusCode,
    // error_buf is owned by the foreign side.
    // - Whatever we are passed, we must never free. This however implies we must be passed
    //   an empty `RustBuffer` otherwise it would leak when we replace it with our own.
    // - On error we will set it to a `RustBuffer` we expect the foreign side to free.
    // We assume initialization, which means we can use `ManuallyDrop` instead of
    // `MaybeUninit`, which avoids unsafe code and clarifies ownership.
    // We must take care to not set this twice to avoid leaking the first `RustBuffer`.
    pub error_buf: ManuallyDrop<RustBuffer>,
}

impl Default for RustCallStatus {
    fn default() -> Self {
        Self {
            code: RustCallStatusCode::Success,
            error_buf: Default::default(),
        }
    }
}

impl RustCallStatus {
    pub fn cancelled() -> Self {
        Self {
            code: RustCallStatusCode::Cancelled,
            error_buf: Default::default(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            code: RustCallStatusCode::UnexpectedError,
            error_buf: ManuallyDrop::new(<String as Lower<UniFfiTag>>::lower(message.into())),
        }
    }
}

/// Result of a FFI call to a Rust function
/// Value is signed to avoid Kotlin's experimental unsigned types.
#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RustCallStatusCode {
    /// Successful call.
    Success = 0,
    /// Expected error, corresponding to the `Result::Err` variant.  [RustCallStatus::error_buf]
    /// will contain the serialized error.
    Error = 1,
    /// Unexpected error.  [RustCallStatus::error_buf] will contain a serialized message string
    UnexpectedError = 2,
    /// Async function cancelled.  [RustCallStatus::error_buf] will be empty and does not need to
    /// be freed.
    ///
    /// This is only returned for async functions and only if the bindings code uses the
    /// [rust_future_cancel] call.
    Cancelled = 3,
}

impl TryFrom<i8> for RustCallStatusCode {
    type Error = i8;

    fn try_from(value: i8) -> Result<Self, i8> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Error),
            2 => Ok(Self::UnexpectedError),
            3 => Ok(Self::Cancelled),
            n => Err(n),
        }
    }
}

/// Error type for Rust scaffolding calls
///
/// This enum represents the fact that there are two ways for a scaffolding call to fail:
/// - Expected errors (the Rust function returned an `Err` value).
/// - Unexpected errors (there was a failure calling the Rust function, for example the failure to
///   lift the arguments).
pub enum RustCallError {
    /// The Rust function returned an `Err` value.
    ///
    /// The associated value is the serialized `Err` value.
    Error(RustBuffer),
    /// There was a failure to call the Rust function, for example a failure to lift the arguments.
    ///
    /// The associated value is a message string for the error.
    InternalError(String),
}

/// Error when trying to lift arguments to pass to the scaffolding call
pub struct LiftArgsError {
    pub arg_name: &'static str,
    pub error: anyhow::Error,
}

impl LiftArgsError {
    pub fn to_internal_error(self) -> RustCallError {
        let LiftArgsError { arg_name, error } = self;

        RustCallError::InternalError(format!(
            "Failed to convert arg '{arg_name}':\n{error:?}"
        ))
    }
}

/// Handle a scaffolding calls
///
/// `callback` is responsible for making the actual Rust call and returning a special result type:
///   - For successful calls, return `Ok(value)`
///   - For errors that should be translated into thrown exceptions in the foreign code, serialize
///     the error into a `RustBuffer`, then return `Ok(buf)`
///   - The success type, must implement `FfiDefault`.
///   - `Return::lower_return` returns `Result<>` types that meet the above criteria>
/// - If the function returns a `Ok` value it will be unwrapped and returned
/// - If the function returns a `Err` value:
///     - `out_status.code` will be set to [RustCallStatusCode::Error].
///     - `out_status.error_buf` will be set to a newly allocated `RustBuffer` containing the error.  The calling
///       code is responsible for freeing the `RustBuffer`
///     - `FfiDefault::ffi_default()` is returned, although foreign code should ignore this value
/// - If the function panics:
///     - `out_status.code` will be set to `CALL_PANIC`
///     - `out_status.error_buf` will be set to a newly allocated `RustBuffer` containing a
///       serialized error message.  The calling code is responsible for freeing the `RustBuffer`
///     - `FfiDefault::ffi_default()` is returned, although foreign code should ignore this value
pub fn rust_call<F, R>(out_status: &mut RustCallStatus, callback: F) -> R
where
    F: panic::UnwindSafe + FnOnce() -> Result<R, RustCallError>,
    R: FfiDefault,
{
    rust_call_with_out_status(out_status, callback).unwrap_or_else(R::ffi_default)
}

/// Make a Rust call and update `RustCallStatus` based on the result.
///
/// If the call succeeds this returns Some(v) and doesn't touch out_status
/// If the call fails (including Err results), this returns None and updates out_status
///
/// This contains the shared code between `rust_call` and `rustfuture::do_wake`.
pub(crate) fn rust_call_with_out_status<F, R>(
    out_status: &mut RustCallStatus,
    callback: F,
) -> Option<R>
where
    F: panic::UnwindSafe + FnOnce() -> Result<R, RustCallError>,
{
    let result = panic::catch_unwind(callback);
    match result {
        // Happy path.  Note: no need to update out_status in this case because the calling code
        // initializes it to [RustCallStatusCode::Success]
        Ok(Ok(v)) => Some(v),
        // Callback returned an Err.
        Ok(Err(RustCallError::Error(buf))) => {
            out_status.code = RustCallStatusCode::Error;
            *out_status.error_buf = buf;
            None
        }
        Ok(Err(RustCallError::InternalError(msg))) => {
            out_status.code = RustCallStatusCode::UnexpectedError;
            *out_status.error_buf = <String as Lower<UniFfiTag>>::lower(msg);
            None
        }
        // Callback panicked
        Err(cause) => {
            out_status.code = RustCallStatusCode::UnexpectedError;
            // Try to coerce the cause into a RustBuffer containing a String.  Since this code can
            // panic, we need to use a second catch_unwind().
            let message_result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
                // The documentation suggests that it will *usually* be a str or String.
                let message = if let Some(s) = cause.downcast_ref::<&'static str>() {
                    (*s).to_string()
                } else if let Some(s) = cause.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic!".to_string()
                };
                trace!("Caught a panic calling rust code: {:?}", message);
                <String as Lower<UniFfiTag>>::lower(message)
            }));
            if let Ok(buf) = message_result {
                // If this was ever set twice we'd leak the old value - but because this is the only
                // place where it is set, and this is only called once, no leaks should exist in practice.
                *out_status.error_buf = buf;
            }
            // Ignore the error case.  We've done all that we can at this point.  In the bindings
            // code, we handle this by checking if `error_buf` still has an empty `RustBuffer` and
            // using a generic message.
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_util::TestError, Lift, LowerReturn};
    use anyhow::anyhow;

    #[test]
    fn test_rust_call() {
        // Successful call
        let mut status = RustCallStatus::default();
        let return_value = rust_call(&mut status, || {
            <Result<i8, TestError> as LowerReturn<UniFfiTag>>::lower_return(Ok(100))
        });

        assert_eq!(status.code, RustCallStatusCode::Success);
        assert_eq!(return_value, 100);

        // Successful call that returns an Err value
        let mut status = RustCallStatus::default();
        rust_call(&mut status, || {
            <Result<i8, TestError> as LowerReturn<UniFfiTag>>::lower_return(Err(TestError(
                "Error".into(),
            )))
        });
        assert_eq!(status.code, RustCallStatusCode::Error);
        assert_eq!(
            <TestError as Lift<UniFfiTag>>::try_lift(ManuallyDrop::into_inner(status.error_buf))
                .unwrap(),
            TestError("Error".to_owned())
        );

        // Internal error while trying to make the call
        let mut status = RustCallStatus::default();
        rust_call(&mut status, || {
            <Result<i8, TestError> as LowerReturn<UniFfiTag>>::handle_failed_lift(LiftArgsError {
                arg_name: "foo",
                error: anyhow!("invalid handle"),
            })
        });
        assert_eq!(status.code, RustCallStatusCode::UnexpectedError);
        assert_eq!(
            <String as Lift<UniFfiTag>>::try_lift(ManuallyDrop::into_inner(status.error_buf))
                .unwrap(),
            "Failed to convert arg 'foo':\ninvalid handle"
        );

        // Panic inside the call
        let mut status = RustCallStatus::default();
        rust_call(&mut status, || -> Result<i8, RustCallError> {
            panic!("I crashed")
        });
        assert_eq!(status.code, RustCallStatusCode::UnexpectedError);
        assert_eq!(
            <String as Lift<UniFfiTag>>::try_lift(ManuallyDrop::into_inner(status.error_buf))
                .unwrap(),
            "I crashed"
        );
    }
}
