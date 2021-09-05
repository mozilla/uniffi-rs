/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeName, KotlinCodeType};
use crate::interface::types::RecordTypeHandler;
use crate::interface::{ComponentInterface, Record};
use askama::Template;

impl KotlinCodeType for RecordTypeHandler<'_> {
    fn nm(&self) -> String {
        names::class_name(self.name)
    }

    fn declare_code(&self, code_builder: CodeBuilder, ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(KotlinRecord::new(
            ci.get_record_definition(self.name)
                .expect("Record definition not found")
                .clone(),
            ci,
        ))
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "RecordTemplate.kt")]
pub struct KotlinRecord {
    rec: Record,
    contains_object_references: bool,
}

impl KotlinRecord {
    pub fn new(rec: Record, ci: &ComponentInterface) -> Self {
        Self {
            contains_object_references: ci.item_contains_object_references(&rec),
            rec,
        }
    }
}
