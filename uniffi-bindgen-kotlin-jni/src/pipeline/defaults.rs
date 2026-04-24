/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_default(default: general::DefaultValue, context: &Context) -> Result<DefaultValueNode> {
    let default: DefaultValue = default.map_node(context)?;
    Ok(DefaultValueNode {
        default_kt: render_default(&default, context)?,
        default,
    })
}

pub fn map_literal(lit: general::Literal, context: &Context) -> Result<LiteralNode> {
    let lit: Literal = lit.map_node(context)?;
    Ok(LiteralNode {
        lit_kt: render_lit(&lit, context)?,
        lit,
    })
}

fn render_default(default: &DefaultValue, context: &Context) -> Result<String> {
    match &default {
        DefaultValue::Literal(lit) => render_lit(lit, context),
        DefaultValue::Default(type_node) => render_type_default(&type_node.ty, context),
    }
}

fn render_type_default(ty: &Type, context: &Context) -> Result<String> {
    Ok(match ty {
        Type::UInt8 => "0.toUByte()".to_string(),
        Type::Int8 => "0.toByte()".to_string(),
        Type::UInt16 => "0.toUShort()".to_string(),
        Type::Int16 => "0.toUShort()".to_string(),
        Type::UInt32 => "0u".to_string(),
        Type::Int32 => "0".to_string(),
        Type::UInt64 => "0uL".to_string(),
        Type::Int64 => "0L".to_string(),
        Type::Float32 => "0.0f".to_string(),
        Type::Float64 => "0.0".to_string(),
        Type::Boolean => "false".to_string(),
        Type::Bytes => "byteArrayOf()".to_string(),
        Type::String => "\"\"".to_string(),
        Type::Record { .. }
        | Type::Enum { .. }
        | Type::Interface { .. }
        | Type::CallbackInterface { .. } => format!("{}()", types::type_kt(ty, context)?),
        Type::Optional { .. } => "null".to_string(),
        Type::Map { .. } => "emptyMap()".to_string(),
        Type::Sequence { .. } => "emptyList()".to_string(),
        Type::Custom { builtin, .. } => render_type_default(builtin, context)
            .map_err(|_| anyhow::anyhow!("Default values not supported for {ty:?}"))?,
        _ => bail!("Default values not supported for {ty:?}"),
    })
}

fn render_lit(lit: &Literal, context: &Context) -> Result<String> {
    Ok(match lit {
        Literal::Boolean(true) => "true".into(),
        Literal::Boolean(false) => "false".into(),
        Literal::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        Literal::UInt(value, _, type_node) => match &type_node.ty {
            Type::Int8 => format!("{value}.toByte()"),
            Type::UInt8 => format!("{value}.toUByte()"),
            Type::Int16 => format!("{value}.toShort()"),
            Type::UInt16 => format!("{value}.toUShort()"),
            Type::Int32 => format!("{value}"),
            Type::UInt32 => format!("{value}u"),
            Type::Int64 => format!("{value}L"),
            Type::UInt64 => format!("{value}uL"),
            ty => bail!("invalid literal type for Literal::UInt ({ty:?})"),
        },
        Literal::Int(value, _, type_node) => match &type_node.ty {
            Type::Int8 => format!("{value}.toByte()"),
            Type::UInt8 => format!("{value}.toUByte()"),
            Type::Int16 => format!("{value}.toShort()"),
            Type::UInt16 => format!("{value}.toUShort()"),
            Type::Int32 => format!("{value}"),
            Type::UInt32 => format!("{value}u"),
            Type::Int64 => format!("{value}L"),
            Type::UInt64 => format!("{value}uL"),
            ty => bail!("invalid literal type for Literal::Int ({ty:?})"),
        },
        Literal::Float(value, type_node) => {
            let value = if value.contains(".") {
                format!("{value}.0")
            } else {
                value.to_string()
            };
            match &type_node.ty {
                Type::Float32 => format!("{value}f"),
                Type::Float64 => value.to_string(),
                ty => bail!("invalid literal type for Literal::Float ({ty:?})"),
            }
        }
        Literal::Enum(variant, type_node) => match &type_node.ty {
            Type::Enum { name, .. } => format!(
                "{}.{}",
                name.to_upper_camel_case(),
                variant.to_shouty_snake_case(),
            ),
            type_kind => {
                bail!("Invalid type for enum literal: {type_kind:?}")
            }
        },
        Literal::EmptySequence => "emptyList()".to_string(),
        Literal::EmptyMap => "emptyMap".to_string(),
        Literal::None => "null".to_string(),
        Literal::Some { inner } => render_default(inner, context)?,
    })
}
