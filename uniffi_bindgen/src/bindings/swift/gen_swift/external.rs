/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::CodeType;

#[derive(Debug)]
pub struct ExternalCodeType {
    name: String,
}

impl ExternalCodeType {
    pub fn new(name: String) -> Self {
        ExternalCodeType { name }
    }
}

impl CodeType for ExternalCodeType {
    fn type_label(&self) -> String {
        super::SwiftCodeOracle.class_name(&self.name)
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }
}
