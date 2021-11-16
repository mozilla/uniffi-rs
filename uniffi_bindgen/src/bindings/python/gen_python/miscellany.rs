/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, Literal};
use askama::Template;
use paste::paste;
use std::fmt;

#[allow(unused_imports)]
use super::filters;

macro_rules! impl_code_type_for_miscellany {
     ($T:ty, $canonical_name:literal, $template_file:literal) => {
         paste! {
             #[derive(Template)]
             #[template(syntax = "py", escape = "none", path = $template_file)]
             pub struct $T;

             impl $T {
                 fn ffi_converter_name(&self, oracle: &dyn CodeOracle) -> String {
                     format!("FfiConverter{}", self.canonical_name(oracle))
                 }
             }

             impl CodeType for $T  {
                 fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
                     format!("{}", $canonical_name)
                 }

                 fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
                     format!("{}", $canonical_name)
                 }

                 fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
                     unreachable!()
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

                fn coerce(&self, _oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
                    nm.to_string()
                }
             }
         }
     }
 }

impl_code_type_for_miscellany!(TimestampCodeType, "Timestamp", "TimestampHelper.py");

impl_code_type_for_miscellany!(DurationCodeType, "Duration", "DurationHelper.py");
