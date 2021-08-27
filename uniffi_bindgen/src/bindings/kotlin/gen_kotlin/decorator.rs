/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeDeclaration, CodeOracle, CodeType};
use crate::interface::{ComponentInterface, DecoratorObject};
use askama::Template;

use super::filters;
pub struct DecoratorObjectCodeType {
    id: String,
}

impl DecoratorObjectCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for DecoratorObjectCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("Decorator{}", self.type_label(oracle))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "DecoratorObjectTemplate.kt")]
pub struct KotlinDecoratorObject {
    inner: DecoratorObject,
}

impl KotlinDecoratorObject {
    pub fn new(inner: DecoratorObject, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &DecoratorObject {
        &self.inner
    }
}

impl CodeDeclaration for KotlinDecoratorObject {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
