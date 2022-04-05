/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use paste::paste;
use uniffi_bindgen::backend::{CodeOracle, CodeType, Literal};

#[allow(unused_imports)]
use super::filters;

macro_rules! impl_code_type_for_miscellany {
    ($T:ty, $class_name:literal, $canonical_name:literal, $template_file:literal) => {
        paste! {
            #[derive(Template)]
            #[template(syntax = "kt", escape = "none", path = $template_file )]
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                    $class_name.into()
                }

                fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
                   $canonical_name.into()
               }

                fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
                    unreachable!()
                }

                fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                    Some(self.render().unwrap())
                }
            }
        }
    };
}

impl_code_type_for_miscellany!(
    TimestampCodeType,
    "java.time.Instant",
    "Timestamp",
    "TimestampHelper.kt"
);

impl_code_type_for_miscellany!(
    DurationCodeType,
    "java.time.Duration",
    "Duration",
    "DurationHelper.kt"
);
