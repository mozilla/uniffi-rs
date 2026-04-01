/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{decode_jni_string, CachedClass, CachedMethod, JniString};
use jni_sys::*;

static INTERNAL_EXCEPTION: CachedClass = CachedClass::new(c"uniffi/InternalException");

/// Throw an `InternalException`
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn throw_internal_exception(env: *mut JNIEnv, message: JniString) {
    let class = INTERNAL_EXCEPTION.get(env);
    // Safety:
    // Env points to a valid JNIEnv
    // We use the JNI API correctly
    unsafe {
        ((**env).v1_4.ThrowNew)(env, class, message.as_ptr());
    }
}

/// Get the error message for a throwable
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn throwable_get_message(env: *mut JNIEnv, throwable: jthrowable) -> String {
    static METHOD: CachedMethod = CachedMethod::new(
        c"java/lang/Throwable",
        c"getMessage",
        c"()Ljava/lang/String;",
    );
    // Safety:
    // Env points to a valid JNIEnv
    // We use the JNI API correctly
    unsafe {
        let method_id = METHOD.get(env);
        let message = ((**env).v1_4.CallObjectMethodA)(env, throwable, method_id, std::ptr::null());
        if message.is_null() {
            return "NULL".into();
        }
        decode_jni_string(env, message)
            .unwrap_or_else(|e| format!("Error decoding Throwable message: {e}"))
    }
}

/// Extension trait for JNI results
pub trait JniResultExt {
    type Ok;

    /// Map a result value to a anyhow result
    ///
    /// # Safety
    /// env must point to a valid JNIEnv
    unsafe fn to_anyhow_result(
        self,
        env: *mut JNIEnv,
        call_description: &str,
    ) -> anyhow::Result<Self::Ok>;

    /// `eprintln` the exception and return a default value.
    ///
    /// # Safety
    /// env must point to a valid JNIEnv
    unsafe fn warn_on_exception(self, env: *mut JNIEnv, call_description: &str) -> Self::Ok
    where
        Self::Ok: Default;
}

impl<T> JniResultExt for Result<T, jthrowable> {
    type Ok = T;

    unsafe fn to_anyhow_result(
        self,
        env: *mut JNIEnv,
        call_description: &str,
    ) -> anyhow::Result<T> {
        self.map_err(|throwable| {
            ((**env).v1_4.ExceptionClear)(env);
            uniffi::deps::anyhow::anyhow!(
                "Exception calling {call_description}: {}",
                throwable_get_message(env, throwable)
            )
        })
    }

    unsafe fn warn_on_exception(self, env: *mut JNIEnv, call_description: &str) -> T
    where
        T: Default,
    {
        self.unwrap_or_else(|throwable| {
            ((**env).v1_4.ExceptionClear)(env);
            eprintln!(
                "Exception calling {call_description}: {}",
                throwable_get_message(env, throwable)
            );
            T::default()
        })
    }
}
