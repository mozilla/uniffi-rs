/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::backend::{CodeType, Literal};

#[derive(Debug)]
pub struct CustomCodeType {
    name: String,
}

impl CustomCodeType {
    pub fn new(name: String) -> Self {
        CustomCodeType { name }
    }
}

impl CodeType for CustomCodeType {
    fn type_label(&self) -> String {
        self.name.clone()
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn literal(&self, _literal: &Literal) -> String {
        unreachable!("Can't have a literal of a custom type");
    }
}
