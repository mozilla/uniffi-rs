/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::bindings::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{CallbackInterface, ComponentInterface};
use askama::Template;

#[allow(unused_imports)]
use super::filters;
pub struct CallbackInterfaceCodeType {
    id: String,
}

impl CallbackInterfaceCodeType {
    pub fn new(_id: String) -> Self {
        panic!("Callbacks are not supported in Swift yet")
    }

    fn internals(&self, oracle: &dyn CodeOracle) -> String {
        format!("{}Internals", self.canonical_name(oracle))
    }
}

impl CodeType for CallbackInterfaceCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("CallbackInterface{}", self.type_label(oracle))
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn lower(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lower({})", self.internals(oracle), oracle.var_name(nm))
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!(
            "{}.write(into: {}, {})",
            self.internals(oracle),
            oracle.var_name(nm),
            target
        )
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.internals(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read(from: {})", self.internals(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} callback interface is found in CallbackInterfaceTemplate.swift",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(
    syntax = "swift",
    escape = "none",
    path = "CallbackInterfaceTemplate.swift"
)]
pub struct SwiftCallbackInterface {
    inner: CallbackInterface,
}

impl SwiftCallbackInterface {
    pub fn new(inner: CallbackInterface, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &CallbackInterface {
        &self.inner
    }
}

impl CodeDeclaration for SwiftCallbackInterface {
    fn initialization_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        let code_type = CallbackInterfaceCodeType::new(self.inner.name().into());
        Some(format!("{}.register(lib)", code_type.internals(oracle)))
    }

    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        None
    }
}

#[derive(Template)]
#[template(
    syntax = "swift",
    escape = "none",
    path = "CallbackInterfaceRuntime.swift"
)]
pub struct SwiftCallbackInterfaceRuntime {
    is_needed: bool,
}

impl SwiftCallbackInterfaceRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_callback_interface_definitions().is_empty(),
        }
    }
}

impl CodeDeclaration for SwiftCallbackInterfaceRuntime {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        if !self.is_needed {
            None
        } else {
            Some(self.render().unwrap())
        }
    }
}
