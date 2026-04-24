/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    caching::{CachedClass, CachedMethod},
    strings::decode_jni_string,
};
use jni_sys::*;

/// Check if a throwable is a uniffi.CallbackException
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn is_callback_exception(env: *mut JNIEnv, throwable: jthrowable) -> bool {
    static CLASS: CachedClass = CachedClass::new(c"uniffi/CallbackException");
    // Safety:
    // Env points to a valid JNIEnv
    // We use the JNI API correctly
    unsafe {
        let class = CLASS.get(env);
        ((**env).v1_2.IsInstanceOf)(env, throwable, class) == JNI_TRUE
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
        let message = ((**env).v1_2.CallObjectMethodA)(env, throwable, method_id, std::ptr::null());
        if message.is_null() {
            return "NULL".into();
        }
        decode_jni_string(env, message)
            .unwrap_or_else(|e| format!("Error decoding Throwable message: {e}"))
    }
}
