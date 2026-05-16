/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_type_node(type_node: general::TypeNode, context: &Context) -> Result<TypeNode> {
    let ty = &type_node.ty;
    let id = *context
        .type_id_map
        .get(ty)
        .ok_or_else(|| anyhow!("Type missing from type_id_map: {ty:?}"))?;

    Ok(TypeNode {
        is_used_as_error: type_node.is_used_as_error,
        id,
        type_rs: type_rs(ty, context)?,
        type_kt: type_kt(ty, context)?,
        ty: type_node.ty.map_node(context)?,
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
