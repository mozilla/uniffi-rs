/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(default: &mut DefaultValueNode) -> Result<()> {
    default.py_default = render_default(&default.default)?;
    Ok(())
}

pub(super) fn render_default(default: &DefaultValue) -> Result<String> {
    Ok(match default {
        DefaultValue::Default(tn) => match tn.ty {
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
            | Type::CallbackInterface { .. } => format!("{}()", tn.canonical_name),
            Type::Optional { .. } => "None".to_string(),
            Type::Map { .. } => "{}".to_string(),
            Type::Sequence { .. } => "[]".to_string(),
            _ => bail!("Default values not supported for {:?}", tn.ty),
        },
        // We assume the Literal pass has already run, so `py_lit` already has a value.
        DefaultValue::Literal(lit) => lit.py_lit.clone(),
    })
}
