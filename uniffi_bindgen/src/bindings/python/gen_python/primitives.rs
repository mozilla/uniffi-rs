/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;
use crate::{
    bail,
    interface::{DefaultValue, Literal, Radix},
    Result,
};

fn render_literal(literal: &Literal) -> Result<String> {
    Ok(match literal {
        Literal::Boolean(v) => {
            if *v {
                "True".into()
            } else {
                "False".into()
            }
        }
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
        Literal::Float(string, _type_) => string.clone(),

        _ => bail!("Invalid literal {literal:?}"),
    })
}

macro_rules! impl_code_type_for_primitive {
    ($T:ident, $python_name:literal, $canonical_name:literal, $def:literal) => {
        #[derive(Debug)]
        pub struct $T;
        impl CodeType for $T {
            fn type_label(&self) -> String {
                $python_name.into()
            }

            fn canonical_name(&self) -> String {
                $canonical_name.into()
            }

            fn default(&self, default: &DefaultValue) -> Result<String> {
                match default {
                    DefaultValue::Default => Ok($def.into()),
                    DefaultValue::Literal(literal) => render_literal(&literal),
                }
            }
        }
    };
}

impl_code_type_for_primitive!(BooleanCodeType, "bool", "Bool", "False");
impl_code_type_for_primitive!(StringCodeType, "str", "String", "\"\"");
impl_code_type_for_primitive!(BytesCodeType, "bytes", "Bytes", "b\"\"");
impl_code_type_for_primitive!(Int8CodeType, "int", "Int8", "0");
impl_code_type_for_primitive!(Int16CodeType, "int", "Int16", "0");
impl_code_type_for_primitive!(Int32CodeType, "int", "Int32", "0");
impl_code_type_for_primitive!(Int64CodeType, "int", "Int64", "0");
impl_code_type_for_primitive!(UInt8CodeType, "int", "UInt8", "0");
impl_code_type_for_primitive!(UInt16CodeType, "int", "UInt16", "0");
impl_code_type_for_primitive!(UInt32CodeType, "int", "UInt32", "0");
impl_code_type_for_primitive!(UInt64CodeType, "int", "UInt64", "0");
impl_code_type_for_primitive!(Float32CodeType, "float", "Float", "0.0");
impl_code_type_for_primitive!(Float64CodeType, "float", "Double", "0.0");
