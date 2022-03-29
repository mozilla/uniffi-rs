/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_bindgen::backend::{CodeOracle, CodeType, Literal};
use uniffi_bindgen::interface::{types::Type, Radix};
use askama::Template;
use paste::paste;

#[allow(unused_imports)]
use super::filters;

fn render_literal(_oracle: &dyn CodeOracle, literal: &Literal) -> String {
    fn typed_number(type_: &Type, num_str: String) -> String {
        match type_ {
            // Bytes, Shorts and Ints can all be inferred from the type.
            Type::Int8 | Type::Int16 | Type::Int32 => num_str,
            Type::Int64 => format!("{}L", num_str),

            Type::UInt8 | Type::UInt16 | Type::UInt32 => format!("{}u", num_str),
            Type::UInt64 => format!("{}uL", num_str),

            Type::Float32 => format!("{}f", num_str),
            Type::Float64 => num_str,
            _ => panic!("Unexpected literal: {} is not a number", num_str),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{}", v),
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Int(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::Float(string, type_) => typed_number(type_, string.clone()),

        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $template_file:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "kt", escape = "none", path = $template_file )]
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                    $class_name.into()
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal)
                }

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }
            }
        }
    };
}

impl_code_type_for_primitive!(BooleanCodeType, "Boolean", "BooleanHelper.kt");
impl_code_type_for_primitive!(StringCodeType, "String", "StringHelper.kt");
impl_code_type_for_primitive!(Int8CodeType, "Byte", "Int8Helper.kt");
impl_code_type_for_primitive!(Int16CodeType, "Short", "Int16Helper.kt");
impl_code_type_for_primitive!(Int32CodeType, "Int", "Int32Helper.kt");
impl_code_type_for_primitive!(Int64CodeType, "Long", "Int64Helper.kt");
impl_code_type_for_primitive!(UInt8CodeType, "UByte", "UInt8Helper.kt");
impl_code_type_for_primitive!(UInt16CodeType, "UShort", "UInt16Helper.kt");
impl_code_type_for_primitive!(UInt32CodeType, "UInt", "UInt32Helper.kt");
impl_code_type_for_primitive!(UInt64CodeType, "ULong", "UInt64Helper.kt");
impl_code_type_for_primitive!(Float32CodeType, "Float", "Float32Helper.kt");
impl_code_type_for_primitive!(Float64CodeType, "Double", "Float64Helper.kt");
