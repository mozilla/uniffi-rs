/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{CodeDeclaration, CodeOracle, CodeType, Literal};
use crate::interface::{CallbackInterface, ComponentInterface};
use askama::Template;

use super::filters;
pub struct CallbackInterfaceCodeType {
    id: String,
}

impl CallbackInterfaceCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
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
    contains_unsigned_types: bool,
}

impl KotlinCallbackInterface {
    pub fn new(inner: CallbackInterface, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: ci.item_contains_unsigned_types(&inner),
            inner,
        }
    }
    pub fn inner(&self) -> &CallbackInterface {
        &self.inner
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl CodeDeclaration for KotlinCallbackInterface {
    fn initialization_code(&self, oracle: &dyn CodeOracle) -> Option<String> {
        let code_type = CallbackInterfaceCodeType::new(self.inner.name().into());
        Some(format!("{}.register(lib)", code_type.internals(oracle)))
    }

    fn definition_code(&self, _oracle: &dyn CodeOracle) -> Option<String> {
        Some(self.render().unwrap())
    }

    fn import_code(&self, _oracle: &dyn CodeOracle) -> Option<Vec<String>> {
        Some(
            vec![
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
