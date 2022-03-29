/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_bindgen::backend::{CodeDeclaration, CodeOracle, CodeType, Literal, TypeIdentifier};
use uniffi_bindgen::interface::{ComponentInterface, Error};
use askama::Template;
use std::borrow::Borrow;

use super::filters;
pub struct ErrorCodeType {
    id: String,
}

impl ErrorCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for ErrorCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.error_name(&self.id)
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.id)
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} error is found in ErrorTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ErrorTemplate.kt")]
pub struct KotlinError {
    inner: Error,
    contains_object_references: bool,
}

impl KotlinError {
    pub fn new(inner: Error, ci: &ComponentInterface) -> Self {
        Self {
            contains_object_references: ci.item_contains_object_references(&inner),
            inner,
        }
    }
    pub fn inner(&self) -> &Error {
        &self.inner
    }
    pub fn contains_object_references(&self) -> bool {
        self.contains_object_references
    }
}

impl CodeDeclaration for KotlinError {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
