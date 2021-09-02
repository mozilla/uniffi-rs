/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeDeclarations, KotlinCodeName, KotlinCodeType};
use crate::interface::types::ErrorTypeHandler;
use crate::interface::{ComponentInterface, Error, Variant};
use crate::Result;
use anyhow::Context;
use askama::Template;

impl KotlinCodeType for ErrorTypeHandler<'_> {
    fn nm(&self) -> String {
        names::error_name(self.name)
    }

    fn declare_code(
        &self,
        declarations: &mut CodeDeclarations,
        ci: &ComponentInterface,
    ) -> Result<()> {
        declarations.definitions.insert(KotlinError::new(
            ci.get_error_definition(self.name)
                .context("Error definition not found")?
                .clone(),
            ci,
        ))
    }
}

trait KotlineError {
    fn variant_name(&self, variant: &Variant) -> String;
}

impl KotlineError for Error {
    fn variant_name(&self, variant: &Variant) -> String {
        names::error_name(variant.name())
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "ErrorTemplate.kt")]
pub struct KotlinError {
    e: Error,
    contains_object_references: bool,
}

impl KotlinError {
    pub fn new(e: Error, ci: &ComponentInterface) -> Self {
        Self {
            contains_object_references: ci.item_contains_object_references(&e),
            e,
        }
    }
}
