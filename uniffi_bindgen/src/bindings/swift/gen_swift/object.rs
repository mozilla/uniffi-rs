/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

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

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lower()", oracle.var_name(nm))
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!("{}.write(into: {})", oracle.var_name(nm), target)
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read(from: {})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} class is found in ObjectTemplate.swift",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "ObjectTemplate.swift")]
pub struct SwiftObject {
    inner: Object,
}

impl SwiftObject {
    pub fn new(inner: Object, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &Object {
        &self.inner
    }
}

impl CodeDeclaration for SwiftObject {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        None
    }
}

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "ObjectRuntime.swift")]
pub struct SwiftObjectRuntime {
    is_needed: bool,
}

impl SwiftObjectRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_object_definitions().is_empty(),
        }
    }
}

impl CodeDeclaration for SwiftObjectRuntime {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        if self.is_needed {
            Some(self.render().unwrap())
        } else {
            None
        }
    }
}
