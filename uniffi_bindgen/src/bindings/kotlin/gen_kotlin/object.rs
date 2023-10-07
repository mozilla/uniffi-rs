/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    backend::{CodeType, Literal},
    interface::ObjectImpl,
};

#[derive(Debug)]
pub struct ObjectCodeType {
    name: String,
    imp: ObjectImpl,
}

impl ObjectCodeType {
    pub fn new(name: String, imp: ObjectImpl) -> Self {
        Self { name, imp }
    }
}

impl CodeType for ObjectCodeType {

    fn type_label(&self) -> String {
        super::KotlinCodeOracle.class_name(&self.name)
    }

    fn protocol_label(&self) -> String {
        if self.id == "Object" {
            self.type_label()
        } else {
            let mut label = self.type_label();
            let index = self.type_label().find("?").unwrap_or(label.len());
            label.insert_str(index, "Interface");
            label
        }
    }

    fn canonical_name(&self) -> String {
        format!("Type{}", self.name)
    }

    fn literal(&self, _literal: &Literal) -> String {
        unreachable!();
    }

    fn initialization_fn(&self) -> Option<String> {
        match &self.imp {
            ObjectImpl::Struct => None,
            ObjectImpl::Trait => Some(format!("uniffiCallbackInterface{}.register", self.name)),
        }
    }

    fn has_abstraction(&self) -> bool { true }
}
