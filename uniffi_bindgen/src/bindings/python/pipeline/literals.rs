/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::ffi_type_names as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    /// Map FfiTypes to the Python type names
    fn map_literal(lit: prev::Literal) -> Result<String> {
        Ok(match lit {
            prev::Literal::Boolean(true) => "True".to_string(),
            prev::Literal::Boolean(false) => "False".to_string(),
            prev::Literal::String(s) => format!("\"{s}\""),
            // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
            prev::Literal::Int(i, radix, _) => match radix {
                prev::Radix::Octal => format!("int(0o{i:o})"),
                prev::Radix::Decimal => format!("{i}"),
                prev::Radix::Hexadecimal => format!("{i:#x}"),
            },
            prev::Literal::UInt(i, radix, _) => match radix {
                prev::Radix::Octal => format!("0o{i:o}"),
                prev::Radix::Decimal => format!("{i}"),
                prev::Radix::Hexadecimal => format!("{i:#x}"),
            },
            prev::Literal::Float(value, _) => value.clone(),
            prev::Literal::EmptySequence => "[]".to_string(),
            prev::Literal::EmptyMap => "{}".to_string(),
            prev::Literal::None => "None".to_string(),
            prev::Literal::Some { inner } => map_literal(*inner)?,
            prev::Literal::Enum(variant, ty) => match &ty.ty {
                prev::Type::Enum { name, .. } => format!("{name}.{variant}"),
                type_kind => {
                    bail!("Invalid type for enum literal: {type_kind:?}")
                }
            },
        })
    }
}
