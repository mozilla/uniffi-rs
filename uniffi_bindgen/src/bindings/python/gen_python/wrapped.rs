/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, TypeIdentifier};
use askama::Template;
use std::fmt;

use super::filters;

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "WrappedTemplate.py")]
pub struct WrappedCodeType {
    inner: TypeIdentifier,
    outer: TypeIdentifier,
}

impl WrappedCodeType {
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

impl CodeType for WrappedCodeType {
    fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
        self.outer.canonical_name()
    }

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        self.type_label(oracle)
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!(
            "{}._lower({})",
            self.type_label(oracle),
            oracle.var_name(nm)
        )
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!(
            "{}._write({}, {})",
            self.type_label(oracle),
            oracle.var_name(nm),
            target
        )
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}._lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}._read({})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn coerce(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        oracle.find(self.inner()).coerce(oracle, nm)
    }
}
