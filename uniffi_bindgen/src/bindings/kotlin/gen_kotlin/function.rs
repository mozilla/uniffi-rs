/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeName, KotlinCodeType};
use crate::interface::{ComponentInterface, Function};
use askama::Template;

pub(super) trait KotlinCodeFunction {
    /// Name for this type in Kotlin code
    fn nm(&self) -> String;

    /// Add code needed for this type to declarations
    fn declare_code(&self, code_builder: CodeBuilder, ci: &ComponentInterface) -> CodeBuilder;
}

impl KotlinCodeFunction for Function {
    fn nm(&self) -> String {
        names::fn_name(self.name())
    }

    fn declare_code(&self, code_builder: CodeBuilder, _ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(KotlinFunction::new(self.clone()))
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "TopLevelFunctionTemplate.kt")]
pub struct KotlinFunction {
    inner: Function,
}

impl KotlinFunction {
    pub fn new(inner: Function) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &Function {
        &self.inner
    }
}
