/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};
use crate::interface::types::Type;
use askama::Template;
use paste::paste;
use std::fmt;

use super::filters;

fn render_literal(oracle: &dyn CodeOracle, literal: &Literal, inner: &TypeIdentifier) -> String {
    match literal {
        Literal::Null => "None".into(),
        Literal::EmptySequence => "[]".into(),
        Literal::EmptyMap => "{}".into(),

        // For optionals
        _ => oracle.find(inner).literal(oracle, literal),
    }
}

macro_rules! impl_code_type_for_compound {
     ($T:ty, $canonical_name_pattern: literal, $template_file:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "py", escape = "none", path = $template_file)]
            pub struct $T {
                inner: TypeIdentifier,
                outer: TypeIdentifier,
            }

            impl $T {
                pub fn new(inner: TypeIdentifier, outer: TypeIdentifier) -> Self {
                    Self { inner, outer }
                }

                fn inner(&self) -> &TypeIdentifier {
                    &self.inner
                }

                fn outer(&self) -> &TypeIdentifier {
                    &self.outer
                }

                fn ffi_converter_name(&self, oracle: &dyn CodeOracle) -> String {
                    format!("FfiConverter{}", self.canonical_name(oracle))
                }
            }

            impl CodeType for $T  {
                fn type_label(&self, oracle: &dyn CodeOracle) -> String {
                    oracle.find(self.inner()).type_label(oracle)
                }

                fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
                    format!($canonical_name_pattern, oracle.find(self.inner()).canonical_name(oracle))
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal, self.inner())
                }

                fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}._lower({})", self.ffi_converter_name(oracle), oracle.var_name(nm))
                }

                fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                    format!("{}._write({}, {})", self.ffi_converter_name(oracle), oracle.var_name(nm), target)
                }

                fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}._lift({})", self.ffi_converter_name(oracle), nm)
                }

                fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}._read({})", self.ffi_converter_name(oracle), nm)
                }

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }
            }
        }
    }
 }

impl_code_type_for_compound!(OptionalCodeType, "Optional{}", "OptionalTemplate.py");
impl_code_type_for_compound!(SequenceCodeType, "Sequence{}", "SequenceTemplate.py");
impl_code_type_for_compound!(MapCodeType, "Map{}", "MapTemplate.py");
