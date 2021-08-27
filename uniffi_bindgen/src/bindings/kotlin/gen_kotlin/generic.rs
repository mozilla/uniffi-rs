/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType};
pub struct GenericCodeType;

impl GenericCodeType {
    pub fn new() -> Self {
        Self
    }
}

impl CodeType for GenericCodeType {
    fn type_label(&self, _oracle: &dyn CodeOracle) -> String {
        "T".to_string()
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        "Generic".to_string()
    }

    fn type_t_label(&self, _oracle: &dyn CodeOracle, t: &str) -> String {
        t.to_string()
    }
}
