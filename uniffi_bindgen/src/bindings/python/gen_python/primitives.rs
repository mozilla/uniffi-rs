/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, Literal};
use crate::interface::Radix;
use paste::paste;

fn render_literal(_oracle: &dyn CodeOracle, literal: &Literal) -> String {
    match literal {
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

        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal) => {
        paste! {
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                    $class_name.into()
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal)
                }
            }
        }
    };
}

impl_code_type_for_primitive!(BooleanCodeType, "Bool");
impl_code_type_for_primitive!(StringCodeType, "String");
impl_code_type_for_primitive!(BytesCodeType, "Bytes");
impl_code_type_for_primitive!(Int8CodeType, "Int8");
impl_code_type_for_primitive!(Int16CodeType, "Int16");
impl_code_type_for_primitive!(Int32CodeType, "Int32");
impl_code_type_for_primitive!(Int64CodeType, "Int64");
impl_code_type_for_primitive!(UInt8CodeType, "UInt8");
impl_code_type_for_primitive!(UInt16CodeType, "UInt16");
impl_code_type_for_primitive!(UInt32CodeType, "UInt32");
impl_code_type_for_primitive!(UInt64CodeType, "UInt64");
impl_code_type_for_primitive!(Float32CodeType, "Float");
impl_code_type_for_primitive!(Float64CodeType, "Double");
