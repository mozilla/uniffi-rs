/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

/// A pass to handle literals and other default values.
pub fn pass(module: &mut Module) -> Result<()> {
    // literals first, because the Default pass might use the value.
    module.try_visit_mut(|lit: &mut LiteralNode| {
        lit.py_lit = render_literal(&lit.lit)?;
        Ok(())
    })?;

    module.try_visit_mut(|default: &mut DefaultValueNode| {
        default.py_default = render_default(&default.default)?;
        Ok(())
    })
}

pub(super) fn render_default(default: &DefaultValue) -> Result<String> {
    Ok(match default {
        DefaultValue::Default(tn) => match &tn.ty {
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
            | Type::CallbackInterface { .. } => format!("{}()", tn.type_name),
            Type::Optional { .. } => "None".to_string(),
            Type::Map { .. } => "{}".to_string(),
            Type::Sequence { .. } => "[]".to_string(),
            Type::Custom { builtin, .. } => {
                return render_default(&DefaultValue::Default(TypeNode {
                    ty: *builtin.clone(),
                    ..tn.clone()
                }))
                .map_err(|_err| anyhow::anyhow!("Default values not supported for {:?}", tn.ty))
            }
            _ => bail!("Default values not supported for {:?}", tn.ty),
        },
        // We assume the Literal pass has already run, so `py_lit` already has a value.
        DefaultValue::Literal(lit) => lit.py_lit.clone(),
    })
}

fn render_literal(lit: &Literal) -> Result<String> {
    Ok(match lit {
        Literal::Boolean(true) => "True".to_string(),
        Literal::Boolean(false) => "False".to_string(),
        Literal::String(s) => format!("\"{s}\""),
        // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
        Literal::Int(i, radix, _) => match radix {
            Radix::Octal => format!("int(0o{i:o})"),
            Radix::Decimal => format!("{i}"),
            Radix::Hexadecimal => format!("{i:#x}"),
        },
        Literal::UInt(i, radix, _) => match radix {
            Radix::Octal => format!("0o{i:o}"),
            Radix::Decimal => format!("{i}"),
            Radix::Hexadecimal => format!("{i:#x}"),
        },
        Literal::Float(value, _) => value.clone(),
        Literal::EmptySequence => "[]".to_string(),
        Literal::EmptyMap => "{}".to_string(),
        Literal::None => "None".to_string(),
        Literal::Some { inner } => match **inner {
            DefaultValue::Literal(ref lit) => render_literal(&lit.lit)?,
            DefaultValue::Default(..) => render_default(inner)?,
        },
        Literal::Enum(variant, ty) => match &ty.ty {
            Type::Enum { name, .. } => format!("{name}.{variant}"),
            type_kind => {
                bail!("Invalid type for enum literal: {type_kind:?}")
            }
        },
    })
}
