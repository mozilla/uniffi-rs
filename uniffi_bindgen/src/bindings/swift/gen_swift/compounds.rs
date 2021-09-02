/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};
use paste::paste;
use std::fmt;

#[allow(unused_imports)]
use super::filters;

fn render_literal(oracle: &dyn CodeOracle, literal: &Literal, inner: &TypeIdentifier) -> String {
    match literal {
        Literal::Null => "nil".into(),
        Literal::EmptySequence => "[]".into(),
        Literal::EmptyMap => "[:]".into(),

        // For optionals
        _ => oracle.find(inner).literal(oracle, literal),
    }
}

macro_rules! impl_code_type_for_compound {
     ($T:ty, $type_label_pattern:literal, $canonical_name_pattern:literal) => {
        paste! {
            pub struct $T {
                inner: TypeIdentifier,
            }

            impl $T {
                pub fn new(inner: TypeIdentifier, _outer: TypeIdentifier) -> Self {
                    Self { inner }
                }
                fn inner(&self) -> &TypeIdentifier {
                    &self.inner
                }
            }

            impl CodeType for $T  {
                fn type_label(&self, oracle: &dyn CodeOracle) -> String {
                    format!($type_label_pattern, oracle.find(self.inner()).type_label(oracle))
                }

                fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
                    format!($canonical_name_pattern, oracle.find(self.inner()).canonical_name(oracle))
                }

                fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
                    render_literal(oracle, &literal, self.inner())
                }

                fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.lower()", oracle.var_name(nm))
                }

                fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                    format!("{}.write(into: {})", oracle.var_name(nm), target)
                }

                fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.lift({})", self.type_label(oracle), nm)
                }

                fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    format!("{}.read(from: {})", self.type_label(oracle), nm)
                }

                fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
                    Some(
                        format!("// Helper code for {} is found in RustBufferHelper.swift", self.type_label(oracle))
                    )
                }
            }
        }
    }
 }

impl_code_type_for_compound!(OptionalCodeType, "{}?", "Option{}");

impl_code_type_for_compound!(SequenceCodeType, "[{}]", "Sequence{}");

impl_code_type_for_compound!(MapCodeType, "[String: {}]", "Map{}");
