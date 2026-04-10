/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_type_node(type_node: general::TypeNode, context: &Context) -> Result<TypeNode> {
    Ok(TypeNode {
        is_used_as_error: type_node.is_used_as_error,
        has_from_unexpected_callback_error_impl: type_node.has_from_unexpected_callback_error_impl,
        type_rs: type_rs(&type_node.ty, context)?,
        type_kt: type_kt(&type_node.ty, context)?,
        read_fn_rs: read_fn_rs(&type_node.ty, &type_node.canonical_name)?,
        write_fn_rs: write_fn_rs(&type_node.ty, &type_node.canonical_name)?,
        read_fn_kt: read_fn_kt(&type_node.ty, &type_node.canonical_name),
        write_fn_kt: write_fn_kt(&type_node.ty, &type_node.canonical_name),
        ty: type_node.ty.map_node(context)?,
        canonical_name: type_node.canonical_name,
    })
}

fn type_rs(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "u8".into(),
        Type::Int8 => "i8".into(),
        Type::UInt16 => "u16".into(),
        Type::Int16 => "i16".into(),
        Type::UInt32 => "u32".into(),
        Type::Int32 => "i32".into(),
        Type::UInt64 => "u64".into(),
        Type::Int64 => "i64".into(),
        Type::Float32 => "f32".into(),
        Type::Float64 => "f64".into(),
        Type::Boolean => "bool".into(),
        Type::String => "::std::string::String".into(),
        Type::Optional { inner_type } => {
            format!("::std::option::Option<{}>", type_rs(inner_type, context)?)
        }
        Type::Sequence { inner_type } => {
            format!("::std::vec::Vec<{}>", type_rs(inner_type, context)?)
        }
        Type::Map {
            key_type,
            value_type,
        } => {
            format!(
                "::std::collections::HashMap<{}, {}>",
                type_rs(key_type, context)?,
                type_rs(value_type, context)?,
            )
        }
        Type::Record {
            namespace,
            orig_name,
            ..
        }
        | Type::Enum {
            namespace,
            orig_name,
            ..
        }
        | Type::Custom {
            namespace,
            orig_name,
            ..
        } => {
            format!(
                "::{}::{orig_name}",
                context.module_path_for_type(namespace, orig_name)?
            )
        }
        Type::Interface {
            namespace,
            orig_name,
            imp,
            ..
        } => {
            if imp.is_trait_interface() {
                format!(
                    "::std::sync::Arc<dyn ::{}::{orig_name}>",
                    context.module_path_for_type(namespace, orig_name)?
                )
            } else {
                format!(
                    "::std::sync::Arc<::{}::{orig_name}>",
                    context.module_path_for_type(namespace, orig_name)?
                )
            }
        }
        Type::CallbackInterface {
            namespace,
            orig_name,
            ..
        } => {
            format!(
                "::std::boxed::Box<dyn ::{}::{orig_name}>",
                context.module_path_for_type(namespace, orig_name)?
            )
        }
        _ => todo!(),
    })
}

pub fn type_kt(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "UByte".into(),
        Type::Int8 => "Byte".into(),
        Type::UInt16 => "UShort".into(),
        Type::Int16 => "Short".into(),
        Type::UInt32 => "UInt".into(),
        Type::Int32 => "Int".into(),
        Type::UInt64 => "ULong".into(),
        Type::Int64 => "Long".into(),
        Type::Float32 => "Float".into(),
        Type::Float64 => "Double".into(),
        Type::Boolean => "Boolean".into(),
        Type::String => "String".into(),
        Type::Optional { inner_type } => {
            format!("{}?", type_kt(inner_type, context)?)
        }
        Type::Sequence { inner_type } => {
            format!("List<{}>", type_kt(inner_type, context)?)
        }
        Type::Map {
            key_type,
            value_type,
        } => {
            format!(
                "Map<{}, {}>",
                type_kt(key_type, context)?,
                type_kt(value_type, context)?,
            )
        }
        Type::Record {
            namespace, name, ..
        }
        | Type::Enum {
            namespace, name, ..
        }
        | Type::Interface {
            namespace, name, ..
        }
        | Type::CallbackInterface {
            namespace, name, ..
        }
        | Type::Custom {
            namespace, name, ..
        } => {
            format!(
                "{}.{}",
                context.package_name(namespace)?,
                names::class_name_kt(name, context.types_used_as_error.contains(&ty)),
            )
        }
        _ => todo!(),
    })
}

pub fn read_fn_rs(ty: &Type, canonical_name: &str) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "uniffi::FfiBufferCursor::read_u8".into(),
        Type::Int8 => "uniffi::FfiBufferCursor::read_i8".into(),
        Type::UInt16 => "uniffi::FfiBufferCursor::read_u16".into(),
        Type::Int16 => "uniffi::FfiBufferCursor::read_i16".into(),
        Type::UInt32 => "uniffi::FfiBufferCursor::read_u32".into(),
        Type::Int32 => "uniffi::FfiBufferCursor::read_i32".into(),
        Type::UInt64 => "uniffi::FfiBufferCursor::read_u64".into(),
        Type::Int64 => "uniffi::FfiBufferCursor::read_i64".into(),
        Type::Float32 => "uniffi::FfiBufferCursor::read_f32".into(),
        Type::Float64 => "uniffi::FfiBufferCursor::read_f64".into(),
        Type::Boolean => "uniffi::FfiBufferCursor::read_bool".into(),
        Type::String => "uniffi::FfiBufferCursor::read_string".into(),
        Type::Optional { .. } | Type::Sequence { .. } | Type::Map { .. } => {
            format!("uniffi_read_compound_{}", canonical_name.to_snake_case(),)
        }
        Type::Record { namespace, .. }
        | Type::Enum { namespace, .. }
        | Type::Interface { namespace, .. }
        | Type::CallbackInterface { namespace, .. }
        | Type::Custom { namespace, .. } => {
            format!(
                "uniffi_read_type_{}_{}",
                namespace.to_snake_case(),
                canonical_name.to_snake_case(),
            )
        }
        _ => todo!(),
    })
}

pub fn write_fn_rs(ty: &Type, canonical_name: &str) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "uniffi::FfiBufferCursor::write_u8".into(),
        Type::Int8 => "uniffi::FfiBufferCursor::write_i8".into(),
        Type::UInt16 => "uniffi::FfiBufferCursor::write_u16".into(),
        Type::Int16 => "uniffi::FfiBufferCursor::write_i16".into(),
        Type::UInt32 => "uniffi::FfiBufferCursor::write_u32".into(),
        Type::Int32 => "uniffi::FfiBufferCursor::write_i32".into(),
        Type::UInt64 => "uniffi::FfiBufferCursor::write_u64".into(),
        Type::Int64 => "uniffi::FfiBufferCursor::write_i64".into(),
        Type::Float32 => "uniffi::FfiBufferCursor::write_f32".into(),
        Type::Float64 => "uniffi::FfiBufferCursor::write_f64".into(),
        Type::Boolean => "uniffi::FfiBufferCursor::write_bool".into(),
        Type::String => "uniffi::FfiBufferCursor::write_string".into(),
        Type::Optional { .. } | Type::Sequence { .. } | Type::Map { .. } => {
            format!("uniffi_write_compound_{}", canonical_name.to_snake_case(),)
        }
        Type::Record { namespace, .. }
        | Type::Enum { namespace, .. }
        | Type::Interface { namespace, .. }
        | Type::CallbackInterface { namespace, .. }
        | Type::Custom { namespace, .. } => {
            format!(
                "uniffi_write_type_{}_{}",
                namespace.to_snake_case(),
                canonical_name.to_snake_case(),
            )
        }
        _ => todo!(),
    })
}

pub fn read_fn_kt(ty: &Type, canonical_name: &str) -> String {
    match ty {
        Type::UInt8 => "readUByte".into(),
        Type::Int8 => "readByte".into(),
        Type::UInt16 => "readUShort".into(),
        Type::Int16 => "readShort".into(),
        Type::UInt32 => "readUInt".into(),
        Type::Int32 => "readInt".into(),
        Type::UInt64 => "readULong".into(),
        Type::Int64 => "readLong".into(),
        Type::Float32 => "readFloat".into(),
        Type::Float64 => "readDouble".into(),
        Type::Boolean => "readBool".into(),
        Type::String => "readString".into(),
        Type::Optional { .. } | Type::Sequence { .. } | Type::Map { .. } => {
            format!("readCompound{}", canonical_name.to_upper_camel_case(),)
        }
        Type::Record { namespace, .. }
        | Type::Enum { namespace, .. }
        | Type::Interface { namespace, .. }
        | Type::CallbackInterface { namespace, .. }
        | Type::Custom { namespace, .. } => {
            format!(
                "readType{}{}",
                namespace.to_upper_camel_case(),
                canonical_name.to_upper_camel_case()
            )
        }
        _ => todo!(),
    }
}

pub fn write_fn_kt(ty: &Type, canonical_name: &str) -> String {
    match ty {
        Type::UInt8 => "writeUByte".into(),
        Type::Int8 => "writeByte".into(),
        Type::UInt16 => "writeUShort".into(),
        Type::Int16 => "writeShort".into(),
        Type::UInt32 => "writeUInt".into(),
        Type::Int32 => "writeInt".into(),
        Type::UInt64 => "writeULong".into(),
        Type::Int64 => "writeLong".into(),
        Type::Float32 => "writeFloat".into(),
        Type::Float64 => "writeDouble".into(),
        Type::Boolean => "writeBool".into(),
        Type::String => "writeString".into(),
        Type::Optional { .. } | Type::Sequence { .. } | Type::Map { .. } => {
            format!("writeCompound{}", canonical_name.to_upper_camel_case(),)
        }
        Type::Record { namespace, .. }
        | Type::Enum { namespace, .. }
        | Type::Interface { namespace, .. }
        | Type::CallbackInterface { namespace, .. }
        | Type::Custom { namespace, .. } => {
            format!(
                "writeType{}{}",
                namespace.to_upper_camel_case(),
                canonical_name.to_upper_camel_case()
            )
        }
        _ => todo!(),
    }
}
