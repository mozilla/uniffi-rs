/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::backend::{CodeBuilder, CodeOracle, CodeType, Literal};
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

    fn ffi_converter_name(&self, oracle: &dyn CodeOracle) -> String {
        format!("FfiConverter{}", self.canonical_name(oracle))
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
        format!(
            "{}.lower({})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm)
        )
    }

    fn write(
        &self,
        oracle: &dyn CodeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!(
            "{}.write({}, {})",
            self.ffi_converter_name(oracle),
            oracle.var_name(nm),
            target
        )
    }

    fn lift(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.ffi_converter_name(oracle), nm)
    }

    fn read(&self, oracle: &dyn CodeOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read({})", self.ffi_converter_name(oracle), nm)
    }

    fn build_code(
        &self,
        oracle: &dyn CodeOracle,
        builder: &mut CodeBuilder,
        ci: &ComponentInterface,
    ) {
        builder
            .add_imports(vec![
                "import java.util.concurrent.locks.ReentrantLock".to_owned(),
                "import kotlin.concurrent.withLock".to_owned(),
            ])
            .add_code_block(KotlinCallbackInterface::new(
                ci.get_callback_interface_definition(&self.id)
                    .unwrap()
                    .clone(),
            ))
            .add_code_block(KotlinCallbackInterfaceRuntime)
            .add_initialization_code(format!("{}.register(lib)", self.ffi_converter_name(oracle)));
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceTemplate.kt")]
pub struct KotlinCallbackInterface {
    inner: CallbackInterface,
}

impl KotlinCallbackInterface {
    pub fn new(inner: CallbackInterface) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &CallbackInterface {
        &self.inner
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceRuntime.kt")]
pub struct KotlinCallbackInterfaceRuntime;
