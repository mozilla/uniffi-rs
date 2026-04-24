/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;

use jni_sys::*;

/// Attach the current thread to the JVM and run a closure
///
/// This is used to implement callback interfaces.
///
/// The closure must not return any JNI objects,
/// since their lifetime ends before this function returns.
///
/// # Safety
///
/// jvm must point to a valid JavaVM
pub unsafe fn attach_current_thread<F, T>(jvm: *mut JavaVM, f: F) -> T
where
    F: FnOnce(*mut JNIEnv) -> T,
{
    let mut env: *mut JNIEnv = ::std::ptr::null_mut();
    // Safety:
    // We're  using the JNI API correctly
    unsafe {
        let attach_result = ((**jvm).v1_2.AttachCurrentThread)(
            jvm,
            std::ptr::from_mut(&mut env).cast::<*mut c_void>(),
            ::std::ptr::null_mut(),
        );
        if attach_result != JNI_OK {
            panic!("AttachCurrentThread failed");
        }

        // Create a new local frame, needed to generate JNI references.
        // Create capacity for 16 references, which is what JNA does
        if ((**env).v1_2.PushLocalFrame)(env, 16) != 0 {
            panic!("Out of memory: PushLocalFrame failed");
        }
        let closure_result = f(env);
        ((**env).v1_2.PopLocalFrame)(env, std::ptr::null_mut());
        closure_result
    }
}
