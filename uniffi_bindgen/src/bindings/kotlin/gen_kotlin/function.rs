/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeDeclaration, CodeOracle};
use crate::interface::{ComponentInterface, Function};
use askama::Template;

use super::filters;

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "TopLevelFunctionTemplate.kt")]
pub struct KotlinFunction {
    inner: Function,
    contains_unsigned_types: bool,
}

impl KotlinFunction {
    pub fn new(inner: Function, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: ci.item_contains_unsigned_types(&inner),
            inner,
        }
    }
    pub fn inner(&self) -> &Function {
        &self.inner
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl CodeDeclaration for KotlinFunction {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
