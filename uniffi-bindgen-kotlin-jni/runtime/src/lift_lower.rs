/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use jni_sys::*;

use super::{decode_jni_string, JniString};

// Lift/Lower functions
//
// This convert between signed JVM primitives and their unsigned counterparts
// These all share the same signature to which simplifies the template code.

/// Define a lift/lower functions for cases where we don't need to do any conversion or only need to
/// do a signed <> unsigned conversion.
macro_rules! define_simple_lift_lower {
    ($lift_name:ident, $lower_name:ident, $ty:ty, $jni_ty:ty) => {
        pub fn $lift_name(_: *mut JNIEnv, value: $jni_ty) -> Result<$ty> {
            Ok(value as $ty)
        }

        pub fn $lower_name(_: *mut JNIEnv, value: $ty) -> Result<$jni_ty> {
            Ok(value as $jni_ty)
        }
    };
}

define_simple_lift_lower!(lift_u8, lower_u8, u8, i8);
define_simple_lift_lower!(lift_i8, lower_i8, i8, i8);
define_simple_lift_lower!(lift_u16, lower_u16, u16, i16);
define_simple_lift_lower!(lift_i16, lower_i16, i16, i16);
define_simple_lift_lower!(lift_u32, lower_u32, u32, i32);
define_simple_lift_lower!(lift_i32, lower_i32, i32, i32);
define_simple_lift_lower!(lift_u64, lower_u64, u64, i64);
define_simple_lift_lower!(lift_i64, lower_i64, i64, i64);
define_simple_lift_lower!(lift_f32, lower_f32, f32, f32);
define_simple_lift_lower!(lift_f64, lower_f64, f64, f64);
define_simple_lift_lower!(lift_bool, lower_bool, bool, bool);

/// Lift a String
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_string(env: *mut JNIEnv, value: jstring) -> Result<String> {
    decode_jni_string(env, value)
}

/// Lower a Rust string
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_string(env: *mut JNIEnv, value: String) -> Result<jstring> {
    Ok(JniString::from(value).into_jstring(env))
}
