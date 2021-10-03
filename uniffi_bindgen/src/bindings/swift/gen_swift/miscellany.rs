/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use askama::Template;
use paste::paste;
use std::fmt;

#[allow(unused_imports)]
use super::filters;

macro_rules! impl_code_type_for_miscellany {
     ($T:ty, $class_name:literal, $canonical_name:literal, $imports:expr, $template_file:literal) => {
         paste! {
             #[derive(Template)]
             #[template(syntax = "swift", escape = "none", path = $template_file)]
             pub struct $T;

             impl CodeType for $T  {
                 fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                     $class_name.into()
                 }

                 fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
                     unreachable!()
                 }

                 fn lift(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.lift({})", $class_name, nm)
                 }

                 fn read(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.read(from: {})", $class_name, nm)
                 }

                 fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                     format!("{}.lower()", oracle.var_name(nm))
                 }

                 fn write(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> String {
                     format!("{}.write(into: {})", oracle.var_name(nm), target)
                 }

                 fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
                     Some(self.render().unwrap())
                 }

                 fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
                    let imports: Vec<&str> = $imports;
                    if (!imports.is_empty()) {
                        Some(
                            imports.into_iter().map(|s| s.into()).collect()
                        )
                    } else {
                        None
                    }
                 }
             }
         }
     }
 }

impl_code_type_for_miscellany!(
    TimestampCodeType,
    "Date",
    "Timestamp",
    vec![],
    "TimestampHelper.swift"
);

impl_code_type_for_miscellany!(
    DurationCodeType,
    "TimeInterval",
    "Duration",
    vec![],
    "DurationHelper.swift"
);
