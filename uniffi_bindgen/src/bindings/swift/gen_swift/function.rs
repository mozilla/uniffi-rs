/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeDeclaration, CodeOracle};
use crate::interface::{ComponentInterface, Function};
use askama::Template;

use super::filters;

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "TopLevelFunctionTemplate.swift")]
pub struct SwiftFunction {
    inner: Function,
}

impl SwiftFunction {
    pub fn new(inner: Function, _ci: &ComponentInterface) -> Self {
        Self {
            inner,
        }
    }
    pub fn inner(&self) -> &Function {
        &self.inner
    }
}

impl CodeDeclaration for SwiftFunction {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
