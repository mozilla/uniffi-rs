/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{CachedClass, JniString};
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
