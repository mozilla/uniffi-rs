/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_type_node(type_node: general::TypeNode, context: &Context) -> Result<TypeNode> {
    let ty = &type_node.ty;

    let ffi_types = if let Some(ffi_types) = ffi_types::standard_ffi_type_mapping(&type_node.ty) {
        ffi_types
    } else {
        todo!();
    };

    Ok(TypeNode {
        is_used_as_error: type_node.is_used_as_error,
        id: type_node.id,
        type_rs: type_rs(ty, context)?,
        type_kt: type_kt(ty, context)?,
        ty: type_node.ty.map_node(context)?,
        ffi_types,
    })
}

pub fn type_rs(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "::std::primitive::u8".into(),
        Type::Int8 => "::std::primitive::i8".into(),
        Type::UInt16 => "::std::primitive::u16".into(),
        Type::Int16 => "::std::primitive::i16".into(),
        Type::UInt32 => "::std::primitive::u32".into(),
        Type::Int32 => "::std::primitive::i32".into(),
        Type::UInt64 => "::std::primitive::u64".into(),
        Type::Int64 => "::std::primitive::i64".into(),
        Type::Float32 => "::std::primitive::f32".into(),
        Type::Float64 => "::std::primitive::f64".into(),
        Type::Boolean => "::std::primitive::bool".into(),
        Type::String => "::std::string::String".into(),
        Type::Record { orig_name, .. }
        | Type::Enum { orig_name, .. }
        | Type::Custom { orig_name, .. } => {
            format!("{}::{orig_name}", context.rust_module_path_for_type(ty)?)
        }
        _ => todo!(),
    })
}

pub fn type_kt(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "kotlin.UByte".into(),
        Type::Int8 => "kotlin.Byte".into(),
        Type::UInt16 => "kotlin.UShort".into(),
        Type::Int16 => "kotlin.Short".into(),
        Type::UInt32 => "kotlin.UInt".into(),
        Type::Int32 => "kotlin.Int".into(),
        Type::UInt64 => "kotlin.ULong".into(),
        Type::Int64 => "kotlin.Long".into(),
        Type::Float32 => "kotlin.Float".into(),
        Type::Float64 => "kotlin.Double".into(),
        Type::Boolean => "kotlin.Boolean".into(),
        Type::String => "kotlin.String".into(),
        Type::Record {
            namespace, name, ..
        }
        | Type::Enum {
            namespace, name, ..
        }
        | Type::Custom {
            namespace, name, ..
        } => {
            format!("{}.{name}", context.package_name_for_namespace(namespace)?)
        }
        _ => todo!(),
    })
}

impl TypeNode {
    /// Function to lower this type
    ///
    /// For primitive types, this converts the high-level type into a single FfiType.
    pub fn lower_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi_jni::lower_i8".into(),
            Type::Int16 => "uniffi_jni::lower_i16".into(),
            Type::Int32 => "uniffi_jni::lower_i32".into(),
            Type::Int64 => "uniffi_jni::lower_i64".into(),
            Type::UInt8 => "uniffi_jni::lower_u8".into(),
            Type::UInt16 => "uniffi_jni::lower_u16".into(),
            Type::UInt32 => "uniffi_jni::lower_u32".into(),
            Type::UInt64 => "uniffi_jni::lower_u64".into(),
            Type::Float32 => "uniffi_jni::lower_f32".into(),
            Type::Float64 => "uniffi_jni::lower_f64".into(),
            Type::Boolean => "uniffi_jni::lower_bool".into(),
            Type::String => "uniffi_jni::lower_string".into(),
            Type::Bytes => "uniffi_jni::lower_bytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "uniffi_jni::lower_option_bool".into(),
                Type::Int8 => "uniffi_jni::lower_option_i8".into(),
                Type::UInt8 => "uniffi_jni::lower_option_u8".into(),
                Type::Int16 => "uniffi_jni::lower_option_i16".into(),
                Type::UInt16 => "uniffi_jni::lower_option_u16".into(),
                Type::Int32 => "uniffi_jni::lower_option_i32".into(),
                Type::UInt32 => "uniffi_jni::lower_option_u32".into(),
                Type::Float32 => "uniffi_jni::lower_option_f32".into(),
                Type::Float64 => "uniffi_jni::lower_option_f64".into(),
                Type::String => "uniffi_jni::lower_option_string".into(),
                _ => format!("lower_type_{id}"),
            },
            _ => format!("lower_type_{id}"),
        }
    }

    /// Function to lift this type
    ///
    /// For primitive types, this converts the FfiType back into the high-level type.
    pub fn lift_fn_rs(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "uniffi_jni::lift_i8".into(),
            Type::Int16 => "uniffi_jni::lift_i16".into(),
            Type::Int32 => "uniffi_jni::lift_i32".into(),
            Type::Int64 => "uniffi_jni::lift_i64".into(),
            Type::UInt8 => "uniffi_jni::lift_u8".into(),
            Type::UInt16 => "uniffi_jni::lift_u16".into(),
            Type::UInt32 => "uniffi_jni::lift_u32".into(),
            Type::UInt64 => "uniffi_jni::lift_u64".into(),
            Type::Float32 => "uniffi_jni::lift_f32".into(),
            Type::Float64 => "uniffi_jni::lift_f64".into(),
            Type::Boolean => "uniffi_jni::lift_bool".into(),
            Type::String => "uniffi_jni::lift_string".into(),
            Type::Bytes => "uniffi_jni::lift_bytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "uniffi_jni::lift_option_bool".into(),
                Type::Int8 => "uniffi_jni::lift_option_i8".into(),
                Type::UInt8 => "uniffi_jni::lift_option_u8".into(),
                Type::Int16 => "uniffi_jni::lift_option_i16".into(),
                Type::UInt16 => "uniffi_jni::lift_option_u16".into(),
                Type::Int32 => "uniffi_jni::lift_option_i32".into(),
                Type::UInt32 => "uniffi_jni::lift_option_u32".into(),
                Type::Float32 => "uniffi_jni::lift_option_f32".into(),
                Type::Float64 => "uniffi_jni::lift_option_f64".into(),
                Type::String => "uniffi_jni::lift_option_string".into(),
                _ => format!("lift_type_{id}"),
            },
            _ => format!("lift_type_{id}"),
        }
    }

    /// Function to lower this type
    ///
    /// For primitive types, this converts the high-level type into a single FfiType.
    pub fn lower_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "lowerByte".into(),
            Type::Int16 => "lowerShort".into(),
            Type::Int32 => "lowerInt".into(),
            Type::Int64 => "lowerLong".into(),
            Type::UInt8 => "lowerUByte".into(),
            Type::UInt16 => "lowerUShort".into(),
            Type::UInt32 => "lowerUInt".into(),
            Type::UInt64 => "lowerULong".into(),
            Type::Float32 => "lowerFloat".into(),
            Type::Float64 => "lowerDouble".into(),
            Type::Boolean => "lowerBoolean".into(),
            Type::String => "lowerString".into(),
            Type::Bytes => "lowerBytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "lowerOptionBoolean".into(),
                Type::Int8 => "lowerOptionByte".into(),
                Type::UInt8 => "lowerOptionUByte".into(),
                Type::Int16 => "lowerOptionShort".into(),
                Type::UInt16 => "lowerOptionUShort".into(),
                Type::Int32 => "lowerOptionInt".into(),
                Type::UInt32 => "lowerOptionUInt".into(),
                Type::Float32 => "lowerOptionFloat".into(),
                Type::Float64 => "lowerOptionDouble".into(),
                Type::String => "lowerOptionString".into(),
                _ => format!("lowerType{id}"),
            },
            _ => format!("lowerType{id}"),
        }
    }

    /// Function to lift this type
    ///
    /// For primitive types, this converts the FfiType back into the high-level type.
    pub fn lift_fn_kt(&self) -> String {
        let id = self.id;
        match &self.ty {
            Type::Int8 => "liftByte".into(),
            Type::Int16 => "liftShort".into(),
            Type::Int32 => "liftInt".into(),
            Type::Int64 => "liftLong".into(),
            Type::UInt8 => "liftUByte".into(),
            Type::UInt16 => "liftUShort".into(),
            Type::UInt32 => "liftUInt".into(),
            Type::UInt64 => "liftULong".into(),
            Type::Float32 => "liftFloat".into(),
            Type::Float64 => "liftDouble".into(),
            Type::Boolean => "liftBoolean".into(),
            Type::String => "liftString".into(),
            Type::Bytes => "liftBytes".into(),
            Type::Optional { inner_type } => match &**inner_type {
                Type::Boolean => "liftOptionBoolean".into(),
                Type::Int8 => "liftOptionByte".into(),
                Type::UInt8 => "liftOptionUByte".into(),
                Type::Int16 => "liftOptionShort".into(),
                Type::UInt16 => "liftOptionUShort".into(),
                Type::Int32 => "liftOptionInt".into(),
                Type::UInt32 => "liftOptionUInt".into(),
                Type::Float32 => "liftOptionFloat".into(),
                Type::Float64 => "liftOptionDouble".into(),
                Type::String => "liftOptionString".into(),
                _ => format!("liftType{id}"),
            },
            _ => format!("liftType{id}"),
        }
    }
}
