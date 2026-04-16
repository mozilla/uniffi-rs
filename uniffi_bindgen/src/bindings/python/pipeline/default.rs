/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn can_use_literal(literal: &general::Literal) -> bool {
    match literal {
        general::Literal::Boolean(_)
        | general::Literal::String(_)
        | general::Literal::UInt(_, _, _)
        | general::Literal::Int(_, _, _)
        | general::Literal::Float(_, _)
        | general::Literal::Enum(_, _) => true,
        general::Literal::Some { inner } => match &**inner {
            general::DefaultValue::Literal(inner_lit) => can_use_literal(inner_lit),
            _ => false,
        },
        _ => false,
    }
}

pub fn render_default(default: &general::DefaultValue, context: &Context) -> Result<String> {
    Ok(match default {
        general::DefaultValue::Default(tn) => match &tn.ty {
            Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64 => "0".to_string(),
            Type::Float32 | Type::Float64 => "0.0".to_string(),
            Type::Boolean => "False".to_string(),
            Type::Bytes => "b\"\"".to_string(),
            Type::String => "\"\"".to_string(),
            Type::Record { .. }
            | Type::Enum { .. }
            | Type::Interface { .. }
            | Type::CallbackInterface { .. } => format!("{}()", types::type_name(&tn.ty, context)?),
            Type::Optional { .. } => "None".to_string(),
            Type::Map { .. } => "{}".to_string(),
            Type::Sequence { .. } => "[]".to_string(),
            Type::Custom { builtin, .. } => {
                return render_default(
                    &general::DefaultValue::Default(general::TypeNode {
                        ty: *builtin.clone(),
                        ..tn.clone()
                    }),
                    context,
                )
                .map_err(|_err| anyhow::anyhow!("Default values not supported for {:?}", tn.ty))
            }
            _ => bail!("Default values not supported for {:?}", tn.ty),
        },
        general::DefaultValue::Literal(lit) => render_literal(lit, context)?,
    })
}

pub fn render_literal(lit: &general::Literal, context: &Context) -> Result<String> {
    Ok(match lit {
        general::Literal::Boolean(true) => "True".to_string(),
        general::Literal::Boolean(false) => "False".to_string(),
        general::Literal::String(s) => format!("\"{s}\""),
        // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
        general::Literal::Int(i, radix, _) => match radix {
            Radix::Octal => format!("int(0o{i:o})"),
            Radix::Decimal => format!("{i}"),
            Radix::Hexadecimal => format!("{i:#x}"),
        },
        general::Literal::UInt(i, radix, _) => match radix {
            Radix::Octal => format!("0o{i:o}"),
            Radix::Decimal => format!("{i}"),
            Radix::Hexadecimal => format!("{i:#x}"),
        },
        general::Literal::Float(value, _) => value.clone(),
        general::Literal::EmptySequence => "[]".to_string(),
        general::Literal::EmptyMap => "{}".to_string(),
        general::Literal::None => "None".to_string(),
        general::Literal::Some { inner } => match **inner {
            general::DefaultValue::Literal(ref lit) => render_literal(lit, context)?,
            general::DefaultValue::Default(..) => render_default(inner, context)?,
        },
        general::Literal::Enum(variant, ty) => match &ty.ty {
            Type::Enum { name, .. } => {
                // Assume enum literals are not error types
                let variant = names::non_error_variant_name(variant);
                format!("{name}.{variant}")
            }
            type_kind => {
                bail!("Invalid type for enum literal: {type_kind:?}")
            }
        },
    })
}

pub fn arg_literal(default: &general::DefaultValue, context: &Context) -> Result<String> {
    Ok(match default {
        general::DefaultValue::Default(tn) => match &tn.ty {
            Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64
            | Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::Float32
            | Type::Float64
            | Type::Boolean
            | Type::Bytes
            | Type::String
            | Type::Optional { .. } => render_default(default, context)?,
            _ => "_DEFAULT".to_string(),
        },
        general::DefaultValue::Literal(lit) => {
            if can_use_literal(lit) {
                render_default(default, context)?
            } else {
                "_DEFAULT".to_string()
            }
        }
    })
}
