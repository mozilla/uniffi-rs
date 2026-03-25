/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};

use anyhow::Result;
use jni_sys::JNIEnv;

use crate::{throw_internal_exception, JniString};

/// Perform a Rust call and catch any panics
///
/// On panic or Error, returns Err(error_message).
pub fn rust_call<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T> + UnwindSafe,
{
    match catch_unwind(f) {
        // Successful call
        Ok(Ok(v)) => Ok(v),
        // Failed call
        Ok(Err(e)) => Err(e.to_string()),
        // Rust panic
        Err(payload) => {
            // The `catch_unwind` documentation suggests that the payload will *usually* be a str or String.
            let message = if let Some(s) = payload.downcast_ref::<&'static str>() {
                format!("Rust panic: {s}")
            } else if let Some(s) = payload.downcast_ref::<String>() {
                format!("Rust panic: {s}")
            } else {
                "Rust panic".to_string()
            };
            Err(message)
        }
    }
}

/// Perform a Rust call inside a JNIEnv
///
/// On error/panic, this throws `uniffi.InternalException` and returns a default value.
///
/// # Safety
///
/// env must point to a valid JNIEnv
pub unsafe fn rust_call_with_env<F, T>(env: *mut JNIEnv, f: F) -> T
where
    F: FnOnce(*mut JNIEnv) -> Result<T> + UnwindSafe,
    T: Default,
{
    // Create a copy of `env` that we can move into the closure
    // This is unwind safe because there's no way for a panic
    // to leave the JNIEnv in an invalid state.
    let env2 = AssertUnwindSafe(env);
    match rust_call(|| f(*env2)) {
        Ok(v) => v,
        Err(e) => {
            // Safety:
            // env points to a valid JNIEnv
            // We're using the JNI API correctly
            unsafe {
                throw_internal_exception(env, JniString::from(e));
            }
            T::default()
        }
    }
}
