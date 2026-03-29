/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::panic::{catch_unwind, AssertUnwindSafe, UnwindSafe};

use anyhow::Result;
use jni_sys::JNIEnv;

use crate::{CachedClass, JniString};

static INTERNAL_EXCEPTION: CachedClass = CachedClass::new(c"uniffi/InternalException");

/// Perform a Rust call and catch any panics
///
/// On panic or Error, this throws a `uniffi.InternalException` error and returns a default value.
///
/// # Safety
///
/// env must point to a valid JNIEnv
pub unsafe fn rust_call<F, T>(env: *mut JNIEnv, f: F) -> T
where
    F: FnOnce(*mut JNIEnv) -> Result<T> + UnwindSafe,
    T: Default,
{
    // Create an copy env that we can move into the closure
    // This is unwind safe because the there's no way for a panic
    // to leave the JNIEnv in an invalid.
    let env2 = AssertUnwindSafe(env);

    match catch_unwind(move || f(*env2)) {
        // Successful call
        Ok(Ok(v)) => v,
        // Failed call
        Ok(Err(e)) => {
            let class = INTERNAL_EXCEPTION.get(env);
            unsafe {
                ((**env).v1_2.ThrowNew)(env, class, JniString::from(e.to_string()).as_ptr());
            }
            T::default()
        }
        // Rust panic
        Err(payload) => {
            // The `catch_unwind` documentation suggests that the payload will *usually* be a str or String.
            let message = if let Some(s) = payload.downcast_ref::<&'static str>() {
                JniString::new(format!("Rust panic: {s}"))
            } else if let Some(s) = payload.downcast_ref::<String>() {
                JniString::new(format!("Rust panic: {s}"))
            } else {
                JniString::new("Rust panic".to_string())
            };

            // Safety:
            // env points to a valid JNIEnv
            // We're using the JNI API correctly
            unsafe {
                let class = INTERNAL_EXCEPTION.get(env);
                ((**env).v1_2.ThrowNew)(env, class, message.as_ptr());
            }
            T::default()
        }
    }
}
