/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(lit: &mut LiteralNode) -> Result<()> {
    lit.py_lit = render_literal(&lit.lit)?;
    Ok(())
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
        Literal::Some { inner } => render_literal(inner)?,
        Literal::Enum(variant, ty) => match &ty.ty {
            Type::Enum { name, .. } => format!("{name}.{variant}"),
            type_kind => {
                bail!("Invalid type for enum literal: {type_kind:?}")
            }
        },
    })
}
