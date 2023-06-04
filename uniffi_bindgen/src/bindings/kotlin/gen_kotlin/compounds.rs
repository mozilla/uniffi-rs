/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, CodeTypeDispatch, Literal, TypeIdentifier};
use paste::paste;

fn render_literal(literal: &Literal, inner: &TypeIdentifier) -> String {
    match literal {
        Literal::Null => "null".into(),
        Literal::EmptySequence => "listOf()".into(),
        Literal::EmptyMap => "mapOf()".into(),

        // For optionals
        _ => super::KotlinCodeOracle.find(inner).literal(literal),
    }
}

macro_rules! impl_code_type_for_compound {
     ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
        paste! {
            #[derive(Debug)]
            pub struct $T {
                inner: TypeIdentifier,
            }

            impl $T {
                pub fn new(inner: TypeIdentifier) -> Self {
                    Self { inner }
                }
                fn inner(&self) -> &TypeIdentifier {
                    &self.inner
                }
            }

            impl CodeType for $T  {
                fn type_label(&self) -> String {
                    format!($type_label_pattern, super::KotlinCodeOracle.find(self.inner()).type_label())
                }

                fn canonical_name(&self) -> String {
                    format!($canonical_name_pattern, super::KotlinCodeOracle.find(self.inner()).canonical_name())
                }

                fn literal(&self, literal: &Literal) -> String {
                    render_literal(literal, self.inner())
                }
            }
        }
    }
 }

impl_code_type_for_compound!(OptionalCodeType, "{}?", "Optional{}");
impl_code_type_for_compound!(SequenceCodeType, "List<{}>", "Sequence{}");

#[derive(Debug)]
pub struct MapCodeType {
    key: TypeIdentifier,
    value: TypeIdentifier,
}

impl MapCodeType {
    pub fn new(key: TypeIdentifier, value: TypeIdentifier) -> Self {
        Self { key, value }
    }

    fn key(&self) -> &TypeIdentifier {
        &self.key
    }

    fn value(&self) -> &TypeIdentifier {
        &self.value
    }
}

impl CodeType for MapCodeType {
    fn type_label(&self) -> String {
        format!(
            "Map<{}, {}>",
            self.key()
                .code_type_impl(&super::KotlinCodeOracle)
                .type_label(),
            self.value()
                .code_type_impl(&super::KotlinCodeOracle)
                .type_label(),
        )
    }

    fn canonical_name(&self) -> String {
        format!(
            "Map{}{}",
            self.key()
                .code_type_impl(&super::KotlinCodeOracle)
                .canonical_name(),
            self.value()
                .code_type_impl(&super::KotlinCodeOracle)
                .canonical_name(),
        )
    }

    fn literal(&self, literal: &Literal) -> String {
        render_literal(literal, &self.value)
    }
}
