/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

/// Standard mappings Type -> FFI types
///
/// The standard mapping is implemented for types where we know the mapping beforehand,
/// without needing to look at the type definitions.
pub fn standard_ffi_type_mapping(ty: &Type) -> Option<Vec<FfiType>> {
    match ty {
        Type::Int8 => Some(vec![FfiType::Int8]),
        Type::Int16 => Some(vec![FfiType::Int16]),
        Type::Int32 => Some(vec![FfiType::Int32]),
        Type::Int64 => Some(vec![FfiType::Int64]),
        Type::UInt8 => Some(vec![FfiType::Int8]),
        Type::UInt16 => Some(vec![FfiType::Int16]),
        Type::UInt32 => Some(vec![FfiType::Int32]),
        Type::UInt64 => Some(vec![FfiType::Int64]),
        Type::Float32 => Some(vec![FfiType::Float32]),
        Type::Float64 => Some(vec![FfiType::Float64]),
        Type::Boolean => Some(vec![FfiType::Boolean]),
        Type::String => Some(vec![FfiType::String]),
        Type::Bytes => Some(vec![FfiType::ByteArray]),
        // Interfaces are passed as 64-bit handle
        Type::Interface { .. } | Type::CallbackInterface { .. } => Some(vec![FfiType::Int64]),
        Type::Optional { inner_type } => match &**inner_type {
            // Ints with less than 64-bits get promoted to an `i64`
            // with `i64::MAX` as the None value.
            Type::UInt8 => Some(vec![FfiType::Int64]),
            Type::Int8 => Some(vec![FfiType::Int64]),
            Type::UInt16 => Some(vec![FfiType::Int64]),
            Type::Int16 => Some(vec![FfiType::Int64]),
            Type::UInt32 => Some(vec![FfiType::Int64]),
            Type::Int32 => Some(vec![FfiType::Int64]),
            Type::Boolean => Some(vec![FfiType::Int64]),
            // Floats use a special-cased NaN value for None
            Type::Float32 => Some(vec![FfiType::Int32]),
            Type::Float64 => Some(vec![FfiType::Int64]),
            // Strings/arrays can use `null` as None
            Type::String => Some(vec![FfiType::String]),
            Type::Sequence { inner_type } => match &**inner_type {
                Type::Bytes => Some(vec![FfiType::ByteArray]),
                _ => None,
            },
            // Interface handles use `0` as the None value.
            Type::Interface { .. } | Type::CallbackInterface { .. } => Some(vec![FfiType::Int64]),
            _ => None,
        },
        _ => None,
    }
}

impl FfiType {
    pub fn type_kt(&self) -> &'static str {
        match self {
            Self::Int8 => "Byte",
            Self::Int16 => "Short",
            Self::Int32 => "Int",
            Self::Int64 => "Long",
            Self::Float32 => "Float",
            Self::Float64 => "Double",
            Self::Boolean => "Boolean",
            // String/byte types are nullable so that we efficiently implement Option types by using
            // `null` as the `None` value.  Also, we can use `null` as a default/uninitialized
            // value.
            Self::String => "String?",
            Self::ByteArray => "kotlin.ByteArray?",
        }
    }

    pub fn type_rs(&self) -> &'static str {
        match self {
            Self::Int8 => "i8",
            Self::Int16 => "i16",
            Self::Int32 => "i32",
            Self::Int64 => "i64",
            Self::Float32 => "f32",
            Self::Float64 => "f64",
            Self::Boolean => "bool",
            // JNI uses the `jstring` type, we convert to `String` in the lift/lower functions.
            Self::String => "uniffi_jni::jstring",
            Self::ByteArray => "uniffi_jni::jbyteArray",
        }
    }

    /// Default/uninitialized value
    pub fn default_kt(&self) -> &'static str {
        match self {
            Self::Int8 => "0.toByte()",
            Self::Int16 => "0.toShort()",
            Self::Int32 => "0",
            Self::Int64 => "0L",
            Self::Float32 => "0.0f",
            Self::Float64 => "0.0",
            Self::Boolean => "false",
            Self::String | Self::ByteArray => "null",
        }
    }

    /// String for this type when used in a JNI signature
    pub fn jni_signature(&self) -> &'static str {
        match self {
            Self::Int8 => "B",
            Self::Int16 => "S",
            Self::Int32 => "I",
            Self::Int64 => "J",
            Self::Float32 => "F",
            Self::Float64 => "D",
            Self::Boolean => "Z",
            Self::String => "Ljava/lang/String;",
            Self::ByteArray => "[B",
        }
    }

    /// Field name for the `jvalue` union type
    pub fn jvalue_field(&self) -> &'static str {
        match self {
            Self::Int8 => "b",
            Self::Int16 => "s",
            Self::Int32 => "i",
            Self::Int64 => "j",
            Self::Float32 => "f",
            Self::Float64 => "d",
            Self::Boolean => "z",
            Self::String | Self::ByteArray => "l",
        }
    }
}

impl FfiArgument {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}
