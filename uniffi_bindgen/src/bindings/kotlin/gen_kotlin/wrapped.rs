/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use crate::interface::{Type, ComponentInterface};
use crate::bindings::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use super::filters;

pub struct WrappedCodeType {
    name: String,
    wrapped: Box<dyn CodeType>,
}

impl WrappedCodeType {
    pub fn new(name: String, wrapped: Box<dyn CodeType>) -> Self {
        Self { name, wrapped }
    }
}

impl CodeType for WrappedCodeType {
    fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
        self.name.clone()
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.name)
    }

    fn literal(&self, oracle: &dyn CodeOracle, literal: &Literal) -> String {
        self.wrapped.literal(oracle, literal)
    }
}


#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "WrappedType.kt")]
pub struct KotlinWrappedType {
    type_: Type,
    name: String,
    wrapped_type: Type,
    contains_unsigned_types: bool,
}

impl KotlinWrappedType {
    pub fn new(type_: Type, name: String, wrapped_type: Type, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: ci.item_contains_unsigned_types(&type_),
            type_,
            name,
            wrapped_type,
        }
    }

    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl CodeDeclaration for KotlinWrappedType {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
