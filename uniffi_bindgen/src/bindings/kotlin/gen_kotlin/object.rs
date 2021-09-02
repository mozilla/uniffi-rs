/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeDeclarations, KotlinCodeName, KotlinCodeType};
use crate::interface::types::ObjectTypeHandler;
use crate::interface::{ComponentInterface, Object};
use crate::Result;
use anyhow::Context;
use askama::Template;

impl KotlinCodeType for ObjectTypeHandler<'_> {
    fn nm(&self) -> String {
        names::class_name(self.name)
    }

    fn declare_code(
        &self,
        declarations: &mut CodeDeclarations,
        ci: &ComponentInterface,
    ) -> Result<()> {
        declarations
            .imports
            .insert("java.util.concurrent.atomic.AtomicLong".into());
        declarations
            .imports
            .insert("java.util.concurrent.atomic.AtomicBoolean".into());
        declarations.runtimes.insert(KotlinObjectRuntime)?;
        declarations.definitions.insert(KotlinObject::new(
            ci.get_object_definition(self.name)
                .context("Object definition not found")?
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
