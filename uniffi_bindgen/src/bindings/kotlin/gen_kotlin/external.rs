/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeOracle, CodeType, Literal};

pub struct ExternalCodeType {
    package_name: String,
    name: String,
}

impl ExternalCodeType {
    pub fn new(package_name: String, name: String) -> Self {
        Self { package_name, name }
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
        unreachable!("Can't have a literal of an external type");
    }

    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        Some(vec![
            format!("{}.{}", self.package_name, self.name),
            format!("{}.FfiConverterType{}", self.package_name, self.name),
        ])
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        None
    }
}
