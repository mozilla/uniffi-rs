/* This Source Code Form is subject to the terms of the Mozilla Publie
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeDeclaration, CodeType, Literal};

pub struct ExternalCodeType {
    name: String,
}

impl ExternalCodeType {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl CodeType for ExternalCodeType {
    fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
        self.name.clone()
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.name)
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }
}

pub struct ExternalDefinition {
    name: String,
    crate_name: String,
}

impl ExternalDefinition {
    pub fn new(name: String, crate_name: String) -> Self {
        Self { name, crate_name }
    }
}

impl CodeDeclaration for ExternalDefinition {
    fn import_code(&self, oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        let crate_name = oracle.mod_name(&self.crate_name);
        Some(vec![
            format!("uniffi.{}.{}", crate_name, self.name),
            format!("uniffi.{}.FFIConverterType{}", crate_name, self.name),
            format!("uniffi.{}.RustBuffer as RustBuffer{}", crate_name, oracle.class_name(&crate_name)),
        ])
    }

    fn definition_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        // Generate a type alias so that consumers can import the external type from this package
        let crate_name = oracle.mod_name(&self.crate_name);
        Some(
            format!("@kotlin.ExperimentalUnsignedTypes\ntypealias {} = uniffi.{}.{}\n",
                self.name, crate_name, self.name))
    }
}
