/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{ComponentInterface, Object};
use askama::Template;

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

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("Object{}", self.type_label(oracle))
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
    contains_unsigned_types: bool,
}

impl KotlinObject {
    pub fn new(inner: Object, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: ci.item_contains_unsigned_types(&inner),
            inner,
        }
    }
    pub fn inner(&self) -> &Object {
        &self.inner
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl CodeDeclaration for KotlinObject {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
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
