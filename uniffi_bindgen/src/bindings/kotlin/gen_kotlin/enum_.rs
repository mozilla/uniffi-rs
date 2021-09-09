/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeBuilder, KotlinCodeName, KotlinCodeType};
use crate::codegen::EnumTypeHandler;
use crate::interface::{ComponentInterface, Enum, Literal, Variant};
use askama::Template;

impl KotlinCodeType for EnumTypeHandler<'_> {
    fn nm(&self) -> String {
        names::class_name(self.name)
    }

    fn literal(&self, literal: &Literal) -> String {
        if let Literal::Enum(v, _) = literal {
            // Note: only fieldless enums are currently supported
            format!("{}.{}", self.nm(), names::enum_variant_name(v),)
        } else {
            unreachable!();
        }
    }

    fn declare_code(&self, code_builder: CodeBuilder, ci: &ComponentInterface) -> CodeBuilder {
        code_builder.code_block(KotlinEnum::new(
            ci.get_enum_definition(self.name)
                .expect("Enum definition not found")
                .clone(),
            ci,
        ))
    }
}

trait KotlineEnum {
    fn variant_name(&self, variant: &Variant) -> String;
}

impl KotlineEnum for Enum {
    fn variant_name(&self, variant: &Variant) -> String {
        if self.is_flat() {
            names::enum_variant_name(variant.name())
        } else {
            names::class_name(variant.name())
        }
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "EnumTemplate.kt")]
pub struct KotlinEnum {
    e: Enum,
    contains_object_references: bool,
}

impl KotlinEnum {
    pub fn new(e: Enum, ci: &ComponentInterface) -> Self {
        Self {
            contains_object_references: ci.item_contains_object_references(&e),
            e,
        }
    }
}
