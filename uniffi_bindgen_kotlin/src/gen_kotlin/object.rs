/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use uniffi_bindgen::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use uniffi_bindgen::interface::{ComponentInterface, Object};

// Filters is used by ObjectTemplate.kt, which looks for the filters module here.
use super::filters;
pub struct ObjectCodeType {
    id: String,
}

impl ObjectCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for ObjectCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.id)
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} class is found in ObjectTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectTemplate.kt")]
pub struct KotlinObject {
    inner: Object,
}

impl KotlinObject {
    pub fn new(inner: Object, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &Object {
        &self.inner
    }
}

impl CodeDeclaration for KotlinObject {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        Some(
            vec![
                "java.util.concurrent.atomic.AtomicLong",
                "java.util.concurrent.atomic.AtomicBoolean",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
        )
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectRuntime.kt")]
pub struct KotlinObjectRuntime {
    is_needed: bool,
}

impl KotlinObjectRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_object_definitions().is_empty(),
        }
    }
}

impl CodeDeclaration for KotlinObjectRuntime {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        if self.is_needed {
            Some(self.render().unwrap())
        } else {
            None
        }
    }
}
