/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
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

/// Lift a FFI Buffer
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_buffer(env: *mut JNIEnv, buf: jobject) -> Result<(*mut u8, usize)> {
    let ptr = ((**env).v1_4.GetDirectBufferAddress)(env, buf);
    let capacity = ((**env).v1_4.GetDirectBufferCapacity)(env, buf);
    Ok((ptr.cast(), capacity as usize))
}

/// Lower a FFI Buffer
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_buffer(env: *mut JNIEnv, ptr: *mut u8, capacity: usize) -> Result<jobject> {
    Ok(((**env).v1_4.NewDirectByteBuffer)(
        env,
        ptr.cast(),
        capacity as i64,
    ))
}

/// Lift/lower functions for Option<T> where T is an int small than 64-bits
///
/// We cast these to a `i64` and use `i64::MAX` as the niche value.
macro_rules! define_lift_lower_option_int {
    ($lift_name:ident, $lower_name:ident, $ty:ty) => {
        pub fn $lift_name(_: *mut JNIEnv, value: i64) -> Result<Option<$ty>> {
            Ok(if value == i64::MAX {
                None
            } else {
                Some(value as $ty)
            })
        }

        pub fn $lower_name(_: *mut JNIEnv, value: Option<$ty>) -> Result<i64> {
            Ok(match value {
                None => i64::MAX,
                Some(v) => v as i64,
            })
        }
    };
}

define_lift_lower_option_int!(lift_option_u8, lower_option_u8, u8);
define_lift_lower_option_int!(lift_option_i8, lower_option_i8, i8);
define_lift_lower_option_int!(lift_option_u16, lower_option_u16, u16);
define_lift_lower_option_int!(lift_option_i16, lower_option_i16, i16);
define_lift_lower_option_int!(lift_option_u32, lower_option_u32, u32);
define_lift_lower_option_int!(lift_option_i32, lower_option_i32, i32);

pub fn lift_option_bool(_: *mut JNIEnv, value: i64) -> Result<Option<bool>> {
    Ok(if value == i64::MAX {
        None
    } else {
        Some(value == 1)
    })
}

pub fn lower_option_bool(_: *mut JNIEnv, value: Option<bool>) -> Result<i64> {
    Ok(match value {
        None => i64::MAX,
        Some(v) => v as i64,
    })
}

pub fn lift_option_f32(_: *mut JNIEnv, value: i32) -> Result<Option<f32>> {
    let value = value as u32;
    Ok(if value == 0xFFFF_FFFF {
        None
    } else {
        Some(f32::from_bits(value))
    })
}

pub fn lower_option_f32(_: *mut JNIEnv, value: Option<f32>) -> Result<i32> {
    Ok(match value {
        None => 0xFFFF_FFFF_u32 as i32,
        Some(v) => match v.to_bits() {
            // The float was encoded using our special-cased NaN value.
            // Convert it to the "preferred" NaN value
            0xFFFF_FFFF => 0xFFC0_0000_u32 as i32,
            v => v as i32,
        },
    })
}

pub fn lift_option_f64(_: *mut JNIEnv, value: i64) -> Result<Option<f64>> {
    let value = value as u64;
    Ok(if value == 0xFFFF_FFFF_FFFF_FFFF {
        None
    } else {
        Some(f64::from_bits(value))
    })
}

pub fn lower_option_f64(_: *mut JNIEnv, value: Option<f64>) -> Result<i64> {
    Ok(match value {
        None => 0xFFFF_FFFF_FFFF_FFFF_u64 as i64,
        Some(v) => match v.to_bits() {
            // The float was encoded using our special-cased NaN value.
            // Convert it to the "preferred" NaN value
            0xFFFF_FFFF_FFFF_FFFF => 0xFFF8_0000_u64 as i64,
            v => v as i64,
        },
    })
}

/// Lift an Option<String>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_option_string(env: *mut JNIEnv, value: jstring) -> Result<Option<String>> {
    Ok(if value.is_null() {
        None
    } else {
        Some(decode_jni_string(env, value)?)
    })
}

/// Lower an Option<String>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_option_string(env: *mut JNIEnv, value: Option<String>) -> Result<jstring> {
    Ok(match value {
        None => std::ptr::null_mut(),
        Some(value) => JniString::from(value).into_jstring(env),
    })
}

/// Lift a Vec<u8>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_u8(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<u8>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_u8: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<u8>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<u8>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_u8(env: *mut JNIEnv, value: Vec<u8>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewByteArray)(env, value.len() as i32);
    ((**env).v1_2.SetByteArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i8>(),
    );
    Ok(array)
}

/// Lift a Vec<i8>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_i8(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<i8>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_i8: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<i8>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<i8>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_i8(env: *mut JNIEnv, value: Vec<i8>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewByteArray)(env, value.len() as i32);
    ((**env).v1_2.SetByteArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i8>(),
    );
    Ok(array)
}

/// Lift a Vec<u16>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_u16(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<u16>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_u16: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<u16>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<u16>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_u16(env: *mut JNIEnv, value: Vec<u16>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewShortArray)(env, value.len() as i32);
    ((**env).v1_2.SetShortArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i16>(),
    );
    Ok(array)
}

/// Lift a Vec<i16>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_i16(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<i16>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_i16: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<i16>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<i16>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_i16(env: *mut JNIEnv, value: Vec<i16>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewShortArray)(env, value.len() as i32);
    ((**env).v1_2.SetShortArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i16>(),
    );
    Ok(array)
}

/// Lift a Vec<u32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_u32(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<u32>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_u32: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<u32>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<u32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_u32(env: *mut JNIEnv, value: Vec<u32>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewIntArray)(env, value.len() as i32);
    ((**env).v1_2.SetIntArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i32>(),
    );
    Ok(array)
}

/// Lift a Vec<i32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_i32(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<i32>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_i32: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<i32>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<i32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_i32(env: *mut JNIEnv, value: Vec<i32>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewIntArray)(env, value.len() as i32);
    ((**env).v1_2.SetIntArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i32>(),
    );
    Ok(array)
}

/// Lift a Vec<u64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_u64(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<u64>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_u64: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<u64>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<u64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_u64(env: *mut JNIEnv, value: Vec<u64>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewLongArray)(env, value.len() as i32);
    ((**env).v1_2.SetLongArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i64>(),
    );
    Ok(array)
}

/// Lift a Vec<i64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_i64(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<i64>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_i64: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<i64>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<i64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_i64(env: *mut JNIEnv, value: Vec<i64>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewLongArray)(env, value.len() as i32);
    ((**env).v1_2.SetLongArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<i64>(),
    );
    Ok(array)
}

/// Lift a Vec<f32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_f32(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<f32>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_f32: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<f32>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<f32>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_f32(env: *mut JNIEnv, value: Vec<f32>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewFloatArray)(env, value.len() as i32);
    ((**env).v1_2.SetFloatArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<f32>(),
    );
    Ok(array)
}

/// Lift a Vec<f64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lift_vec_f64(env: *mut JNIEnv, value: jbyteArray) -> Result<Vec<f64>> {
    let len = ((**env).v1_2.GetArrayLength)(env, value);
    let data = ((**env).v1_2.GetPrimitiveArrayCritical)(env, value, std::ptr::null_mut());
    if data.is_null() {
        bail!("lift_vec_f64: GetPrimitiveArrayCritical failed");
    }
    let slice = std::slice::from_raw_parts(data.cast::<f64>(), len as usize);
    let vec = slice.to_vec();
    // JNI_ABORT releases the array data without committing any changes back,
    // which is safe since we only copied the data.
    ((**env).v1_2.ReleasePrimitiveArrayCritical)(env, value, data, JNI_ABORT);
    Ok(vec)
}

/// Lower a Vec<f64>
///
/// # Safety
/// env must point to a valid JNIEnv
pub unsafe fn lower_vec_f64(env: *mut JNIEnv, value: Vec<f64>) -> Result<jbyteArray> {
    let array = ((**env).v1_2.NewDoubleArray)(env, value.len() as i32);
    ((**env).v1_2.SetDoubleArrayRegion)(
        env,
        array,
        0,
        value.len() as i32,
        value.as_ptr().cast::<f64>(),
    );
    Ok(array)
}
