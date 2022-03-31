/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use uniffi_bindgen::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use uniffi_bindgen::interface::{CallbackInterface, ComponentInterface};

use super::filters;
pub struct CallbackInterfaceCodeType {
    id: String,
}

impl CallbackInterfaceCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for CallbackInterfaceCodeType {
    fn type_label(&self, oracle: &dyn CodeOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, _oracle: &dyn CodeOracle) -> String {
        format!("Type{}", self.id)
    }

    fn literal(&self, _oracle: &dyn CodeOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn helper_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} callback interface is found in CallbackInterfaceTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceTemplate.kt")]
pub struct KotlinCallbackInterface {
    inner: CallbackInterface,
}

impl KotlinCallbackInterface {
    pub fn new(inner: CallbackInterface, _ci: &ComponentInterface) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &CallbackInterface {
        &self.inner
    }
}

impl CodeDeclaration for KotlinCallbackInterface {
    fn initialization_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        let code_type = CallbackInterfaceCodeType::new(self.inner.name().into());
        Some(format!(
            "{}.register(lib)",
            code_type.ffi_converter_name(oracle)
        ))
    }

    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn imports(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        Some(
            vec![
                "java.util.concurrent.atomic.AtomicLong",
                "java.util.concurrent.locks.ReentrantLock",
                "kotlin.concurrent.withLock",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
        )
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceRuntime.kt")]
pub struct KotlinCallbackInterfaceRuntime {
    is_needed: bool,
}

impl KotlinCallbackInterfaceRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_callback_interface_definitions().is_empty(),
        }
    }
}

impl CodeDeclaration for KotlinCallbackInterfaceRuntime {
    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        if !self.is_needed {
            None
        } else {
            Some(self.render().unwrap())
        }
    }
}
