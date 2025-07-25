/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;
use crate::{interface::DefaultValue, ComponentInterface};

#[derive(Debug)]
pub struct CustomCodeType {
    name: String,
    builtin: Box<dyn CodeType>,
}

impl CustomCodeType {
    pub fn new(name: String, builtin: Box<dyn CodeType>) -> Self {
        CustomCodeType { name, builtin }
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self, ci: &ComponentInterface) -> String {
        super::KotlinCodeOracle.class_name(ci, &self.name)
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn default(&self, default: &DefaultValue, ci: &ComponentInterface) -> anyhow::Result<String> {
        self.builtin
            .default(default, ci)
            .map_err(|_e| anyhow::anyhow!("Unsupported default value for {}", self.type_label(ci)))
    }
}
