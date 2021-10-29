/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeOracle, CodeType, Literal};
use std::fmt;

pub struct WrappedCodeType {
    name: String,
    prim: Box<dyn CodeType>,
}

impl WrappedCodeType {
    pub fn new(name: String, prim: Box<dyn CodeType>) -> Self {
        WrappedCodeType { name, prim }
    }
}

impl CodeType for WrappedCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        // vars/params/etc must be declared as the primitive type.
        self.prim.type_label(oracle)
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        self.name.clone()
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        // No such thing as a literal wrapped value.
        unreachable!("Can't have a literal of a wrapped object");
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        self.prim.lower(oracle, nm)
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        self.prim.write(oracle, nm, target)
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        self.prim.lift(oracle, nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        self.prim.read(oracle, nm)
    }

    fn helper_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        // We don't want the helper code for the wrapped type, otherwise we
        // will end up with 2 copies of it, which breaks things.
        None
    }
}
