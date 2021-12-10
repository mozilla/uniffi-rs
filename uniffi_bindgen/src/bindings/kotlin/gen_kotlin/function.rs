/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::interface::{ComponentInterface, Function};
use askama::Template;

use super::filters;

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "TopLevelFunctionTemplate.kt")]
pub struct KotlinFunction {
    inner: Function,
}

impl KotlinFunction {
    pub fn new(inner: Function, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &Function {
        &self.inner
    }
}
