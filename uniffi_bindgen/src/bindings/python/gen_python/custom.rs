/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{filters, CustomTypeConfig};
use crate::backend::{CodeDeclaration, CodeOracle, CodeType, TypeIdentifier};
use askama::Template;

pub struct CustomCodeType {
    name: String,
    builtin: TypeIdentifier,
    config: Option<CustomTypeConfig>,
}

impl CustomCodeType {
    pub fn new(name: String, builtin: TypeIdentifier, config: Option<CustomTypeConfig>) -> Self {
        Self {
            name,
            builtin,
            config,
        }
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        match self.config {
            // The consumer provided custom type config, which means we don't know our exact type.  This
            // is fine for python though, let's just use "object" as a placeholder.
            Some(_) => "object".to_string(),
            // No custom type config provided.  We're just an alias for our builtin type.
            None => self.builtin.type_label(oracle),
        }
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.name)
    }

    fn coerce(&self, _oracle: &dyn CodeOracle, nm: &str) -> String {
        nm.to_string()
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "# Helper code for {} is found in CustomType.py",
            self.name,
        ))
    }
}

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "CustomType.py")]
pub struct PythonCustomType {
    name: String,
    builtin: TypeIdentifier,
    config: Option<CustomTypeConfig>,
}

impl PythonCustomType {
    pub fn new(name: String, builtin: TypeIdentifier, config: Option<CustomTypeConfig>) -> Self {
        Self {
            name,
            builtin,
            config,
        }
    }
}

impl CodeDeclaration for PythonCustomType {
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
