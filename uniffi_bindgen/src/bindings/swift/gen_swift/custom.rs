/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{filters, CustomTypeConfig};
use crate::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{FFIType, Type};
use askama::Template;
use std::fmt;

pub struct CustomCodeType {
    name: String,
    builtin: Type,
    config: Option<CustomTypeConfig>,
}

impl CustomCodeType {
    pub fn new(name: String, builtin: Type, config: Option<CustomTypeConfig>) -> Self {
        CustomCodeType {
            name,
            builtin,
            config,
        }
    }

    pub fn ffi_converter_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("FfiConverterType{}", self.name)
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        match &self.config {
            None => self.builtin.type_label(oracle),
            Some(custom_type_config) => custom_type_config
                .type_name
                .clone()
                .unwrap_or_else(|| self.name.clone()),
        }
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        self.name.clone()
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        // No such thing as a literal custom type
        unreachable!("Can't have a literal of a custom type");
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!(
            "{}.lower({})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm)
        )
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!(
            "{}.write({}, {})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm),
            target
        )
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.ffi_converter_name(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read({})", self.ffi_converter_name(oracle), nm)
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} is found in CustomType.py",
            self.name,
        ))
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        match &self.config {
            None => None,
            Some(custom_type_config) => custom_type_config.imports.clone(),
        }
    }
}

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "CustomType.swift")]
pub struct SwiftCustomType {
    name: String,
    builtin: Type,
    config: Option<CustomTypeConfig>,
}

impl SwiftCustomType {
    pub fn new(name: String, builtin: Type, config: Option<CustomTypeConfig>) -> Self {
        SwiftCustomType {
            name,
            builtin,
            config,
        }
    }

    fn type_name(&self, config: &CustomTypeConfig) -> String {
        config
            .type_name
            .clone()
            .unwrap_or_else(|| self.name.clone())
    }

    fn builtin_ffi_type(&self) -> FFIType {
        FFIType::from(&self.builtin)
    }
}

impl CodeDeclaration for SwiftCustomType {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        match &self.config {
            None => None,
            Some(custom_type_config) => custom_type_config.imports.clone(),
        }
    }
}
