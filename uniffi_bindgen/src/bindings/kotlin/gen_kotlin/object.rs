/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeName, KotlinCodeType};
use crate::codegen::ObjectTypeHandler;
use crate::interface::{ComponentInterface, Object};
use askama::Template;

impl KotlinCodeType for ObjectTypeHandler<'_> {
    fn nm(&self) -> String {
        names::class_name(self.name)
    }

    fn declare_code(&self, code_builder: CodeBuilder, ci: &ComponentInterface) -> CodeBuilder {
        code_builder
            .import("java.util.concurrent.atomic.AtomicLong".into())
            .import("java.util.concurrent.atomic.AtomicBoolean".into())
            .code_block(KotlinObjectRuntime)
            .code_block(KotlinObject::new(
                ci.get_object_definition(self.name)
                    .expect("Object definition not found")
                    .clone(),
                ci,
            ))
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "ObjectTemplate.kt")]
pub struct KotlinObject {
    obj: Object,
}

impl KotlinObject {
    pub fn new(obj: Object, _ci: &ComponentInterface) -> Self {
        Self { obj }
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "ObjectRuntime.kt")]
pub struct KotlinObjectRuntime;
