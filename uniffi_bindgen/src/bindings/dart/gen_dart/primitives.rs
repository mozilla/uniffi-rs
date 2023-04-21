/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, Literal};
use crate::interface::{types::Type, Radix};
use paste::paste;

fn render_literal(oracle: &dyn CodeOracle, literal: &Literal) -> String {
    fn typed_number(oracle: &dyn CodeOracle, type_: &Type, num_str: String) -> String {
        match type_ {
            // special case Int32.
            Type::Int32 => num_str,
            // otherwise use constructor e.g. UInt8(x)
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64 =>
            // XXX we should pass in the codetype itself.
            {
                format!("{}({num_str})", oracle.find(type_).type_label(oracle))
            }
            _ => panic!("Unexpected literal: {num_str} is not a number"),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{v}"),
        Literal::String(s) => format!("\"{s}\""),
        Literal::Int(i, radix, type_) => typed_number(
            oracle,
            type_,
            match radix {
                Radix::Octal => format!("0o{i:o}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            oracle,
            type_,
            match radix {
                Radix::Octal => format!("0o{i:o}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        Literal::Float(string, type_) => typed_number(oracle, type_, string.clone()),
        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $dart_type:literal) => {
        paste! {
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                    $class_name.into()
                }

                fn lower(&self, oracle: &dyn CodeOracle) -> String {
                    "noop".into()
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal)
                }
            }
        }
    };
}

impl_code_type_for_primitive!(BooleanCodeType, "Bool", "boolean");
impl_code_type_for_primitive!(StringCodeType, "String", "string");
impl_code_type_for_primitive!(Int8CodeType, "Int8", "int");
impl_code_type_for_primitive!(Int16CodeType, "Int16", "int");
impl_code_type_for_primitive!(Int32CodeType, "Int32", "int");
impl_code_type_for_primitive!(Int64CodeType, "Int64", "int");
impl_code_type_for_primitive!(UInt8CodeType, "Uint8", "int");
impl_code_type_for_primitive!(UInt16CodeType, "Uint16", "int");
impl_code_type_for_primitive!(UInt32CodeType, "Uint32", "int");
impl_code_type_for_primitive!(UInt64CodeType, "Uint64", "int");
impl_code_type_for_primitive!(Float32CodeType, "Float", "float");
impl_code_type_for_primitive!(Float64CodeType, "Double", "double");
