/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;

#[derive(Debug)]
pub struct CustomCodeType {
    name: String,
    builtin: Box<dyn CodeType>,
}

impl CustomCodeType {
    pub fn new(name: String, builtin: Box<dyn CodeType>) -> Self {
        Self { name, builtin }
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self) -> String {
        super::SwiftCodeOracle.class_name(&self.name)
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn default(&self, default: &crate::interface::DefaultValue) -> anyhow::Result<String> {
        self.builtin
            .default(default)
            .map_err(|_e| anyhow::anyhow!("Unsupported default value for {}", self.type_label()))
    }
}
