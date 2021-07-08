/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Low-level support for calling rust functions
//!
//! This module helps the scaffolding code make calls to rust functions and pass back the result to the FFI bindings code.
//!
//! It handles:
//!    - Catching panics
//!    - Adapting `Result<>` types into either a return value or an error

use crate::{RustBuffer, ViaFfi};
use anyhow::Result;
use ffi_support::IntoFfi;
use std::panic;

/// Represents the success/error of a rust call
///
/// ## Usage
///
/// - The consumer code creates a `RustCallStatus` with an empty `RustBuffer` and `CALL_SUCCESS`
///   (0) as the status code
/// - A pointer to this object is passed to the rust FFI function.  This is an
///   "out parameter" which will be updated with any error that occurred during the function's
///   execution.
/// - After the call, if code is `CALL_ERROR` then `error_buf` will be updated to contain
///   the serialized error object.   The consumer is responsible for freeing `error_buf`.
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
///
/// #### The `code` field.
///
///  - `CALL_SUCESS` (0) for successful calls
///  - `CALL_ERROR` (1) for calls that returned an `Err` value
///  - `CALL_PANIC` (2) for calls that panicked
///
/// #### The `error_buf` field.
///
/// - For `CALL_ERROR` this is a `RustBuffer` with the serialized error.  The consumer code is
///   responsible for freeing this `RustBuffer`.
#[repr(C)]
pub struct RustCallStatus {
    pub code: i8, // Signed because unsigned types are experimental in Kotlin
    pub error_buf: RustBuffer,
}

#[allow(dead_code)] // CALL_SUCCESS is set by the calling code
const CALL_SUCCESS: i8 = 0;
const CALL_ERROR: i8 = 1;
const CALL_PANIC: i8 = 2;

// Generalized rust call handling function
fn make_call<F, R>(out_status: &mut RustCallStatus, callback: F) -> R::Value
where
    F: panic::UnwindSafe + FnOnce() -> Result<R, RustBuffer>,
    R: IntoFfi,
{
    // The closure below is not generally unwind safe because we mutate out_status.  However, in
    // this case it's safe because:
    //
    //   - If there's a panic, we overwrite any value in `code` with CALL_PANIC.
    //   - After we set `error_buf`, no more panics are possible
    panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // We should call:
        //
        // init_panic_handling_once();
        //
        // This is called in ffi-support::call_with_result_impl().  The current plan is to make it
        // `pub` in ffi-support and call it from here.
        match callback() {
            Ok(v) => {
                // No need to update out_status in this case because the calling code initializes
                // it to 0
                v.into_ffi_value()
            }
            Err(buf) => {
                out_status.code = CALL_ERROR;
                out_status.error_buf = buf;
                R::ffi_default()
            }
        }
    }))
    .unwrap_or_else(|_| {
        out_status.code = CALL_PANIC;
        R::ffi_default()
    })
}

/// Wrap a rust function call and return the result directly
///
/// - If the function succeeds then the function's return value will be returned to the outer code
/// - If the function panics:
///     - `out_status.code` will be set to `CALL_PANIC`
///     - the return value is undefined
pub fn call_with_output<F, R>(out_status: &mut RustCallStatus, callback: F) -> R::Value
where
    F: panic::UnwindSafe + FnOnce() -> R,
    R: IntoFfi,
{
    return make_call(out_status, || Ok(callback()))
}

/// Wrap a rust function call that returns a `Result<>`
///
/// - If the function returns an `Ok` value it will be unwrapped and returned
/// - If the function returns an `Err`:
///     - `out_status.code` will be set to `CALL_ERROR`
///     - `out_status.error_buf` will be set to a newly allocated `RustBuffer` containing the error.  The calling
///       code is responsible for freeing the `RustBuffer`
///     - the return value is undefined
/// - If the function panics:
///     - `out_status.code` will be set to `CALL_PANIC`
///     - the return value is undefined
pub fn call_with_result<F, R, E>(out_status: &mut RustCallStatus, callback: F) -> R::Value
where
    F: panic::UnwindSafe + FnOnce() -> Result<R, E>,
    E: ViaFfi<FfiType = RustBuffer>,
    R: IntoFfi,
{
    return make_call(out_status, || callback().map_err(|e| e.lower()))
}

#[cfg(test)]
mod test {
    use super::*;

    fn function(a: u8) -> i8 {
        match a {
            0 => 100,
            x => panic!("Unexpected value: {}", x),
        }
    }

    fn create_call_status() -> RustCallStatus {
        RustCallStatus {
            code: 0,
            error_buf: RustBuffer::new(),
        }
    }

    #[test]
    fn test_call_with_output() {
        let mut status = create_call_status();
        let return_value = call_with_output(&mut status, || function(0));
        assert_eq!(status.code, CALL_SUCCESS);
        assert_eq!(return_value, 100);

        call_with_output(&mut status, || function(1));
        assert_eq!(status.code, CALL_PANIC);
    }

    fn function_with_result(a: u8) -> Result<i8, String> {
        match a {
            0 => Ok(100),
            1 => Err("Error".to_owned()),
            x => panic!("Unexpected value: {}", x),
        }
    }

    #[test]
    fn test_call_with_result() {
        let mut status = create_call_status();
        let return_value = call_with_result(&mut status, || function_with_result(0));
        assert_eq!(status.code, CALL_SUCCESS);
        assert_eq!(return_value, 100);

        call_with_result(&mut status, || function_with_result(1));
        assert_eq!(status.code, CALL_ERROR);
        assert_eq!(
            <String as ViaFfi>::try_lift(status.error_buf).unwrap(),
            "Error".to_owned()
        );

        let mut status = create_call_status();
        call_with_result(&mut status, || function_with_result(2));
        assert_eq!(status.code, CALL_PANIC);
    }
}
