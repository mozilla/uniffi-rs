/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::filters;
use askama::Template;
use paste::paste;
use std::borrow::Borrow;
use uniffi_bindgen::backend::{CodeOracle, CodeType, Literal, TypeIdentifier};
use uniffi_bindgen::interface::types::Type;

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
     ($T:ty, $canonical_name_pattern: literal, $template_file:literal, $coerce_code:expr) => {
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

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }

                fn coerce(&self, oracle: &dyn CodeOracle, nm: &str) -> String {
                    $coerce_code(self, oracle, nm)
                }
            }
        }
    }
 }

impl_code_type_for_compound!(
    OptionalCodeType,
    "Optional{}",
    "OptionalTemplate.py",
    optional_coerce
);
fn optional_coerce(this: &OptionalCodeType, oracle: &dyn CodeOracle, nm: &str) -> String {
    format!(
        "(None if {} is None else {})",
        nm,
        oracle.find(this.inner()).coerce(oracle, nm)
    )
}

impl_code_type_for_compound!(
    SequenceCodeType,
    "Sequence{}",
    "SequenceTemplate.py",
    sequence_coerce
);
fn sequence_coerce(this: &SequenceCodeType, oracle: &dyn CodeOracle, nm: &str) -> String {
    format!(
        "list({} for x in {})",
        oracle.find(this.inner()).coerce(oracle, &"x".to_string()),
        nm
    )
}
impl_code_type_for_compound!(MapCodeType, "Map{}", "MapTemplate.py", map_coerce);
fn map_coerce(this: &MapCodeType, oracle: &dyn CodeOracle, nm: &str) -> String {
    format!(
        "dict(({}, {}) for (k, v) in {}.items())",
        oracle.find(&Type::String).coerce(oracle, &"k".to_string()),
        oracle.find(this.inner()).coerce(oracle, &"v".to_string()),
        nm
    )
}
