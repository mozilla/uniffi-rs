/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use jni_sys::{jstring, JNIEnv};
use simd_cesu8::mutf8;

/// Decode/convert a `jstring` into a Rust String
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn decode_jni_string(env: *mut JNIEnv, string: jstring) -> Result<String> {
    // Safety:
    // We're using the JNI API correctly
    unsafe {
        let len = ((**env).v1_2.GetStringUTFLength)(env, string);
        let data = ((**env).v1_2.GetStringUTFChars)(env, string, std::ptr::null_mut());
        let bytes: &[u8] = std::slice::from_raw_parts(data.cast::<u8>(), len as usize);
        let rust_string = mutf8::decode(bytes).map(|bytes| bytes.to_string());
        ((**env).v1_2.ReleaseStringUTFChars)(env, string, data);
        Ok(rust_string?)
    }
}
