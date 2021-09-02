/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{names, CodeDeclarations, KotlinCodeName, KotlinCodeType};
use crate::interface::types::CallbackInterfaceTypeHandler;
use crate::interface::types::NewCodeType;
use crate::interface::{CallbackInterface, ComponentInterface, Method};
use crate::Result;
use anyhow::Context;
use askama::Template;

fn internals_name(cbi: &impl NewCodeType) -> String {
    format!("{}Internals", cbi.canonical_name())
}

fn interface_impl_name(cbi: &impl NewCodeType) -> String {
    format!("{}FFI", cbi.canonical_name())
}

impl KotlinCodeType for CallbackInterfaceTypeHandler<'_> {
    fn nm(&self) -> String {
        names::class_name(self.name)
    }

    fn lower(&self, nm: &str) -> String {
        format!("{}.lower({})", internals_name(self), names::var_name(nm))
    }

    fn write(&self, nm: &str, target: &str) -> String {
        format!(
            "{}.write({}, {})",
            internals_name(self),
            names::var_name(nm),
            target
        )
    }

    fn lift(&self, nm: &str) -> String {
        format!("{}.lift({})", internals_name(self), nm)
    }

    fn read(&self, nm: &str) -> String {
        format!("{}.read({})", internals_name(self), nm)
    }

    fn declare_code(
        &self,
        declarations: &mut CodeDeclarations,
        ci: &ComponentInterface,
    ) -> Result<()> {
        declarations
            .imports
            .insert("java.util.concurrent.locks.ReentrantLock".into());
        declarations
            .imports
            .insert("kotlin.concurrent.withLock".into());
        declarations
            .runtimes
            .insert(KotlinCallbackInterfaceRuntime)?;
        declarations
            .definitions
            .insert(KotlinCallbackInterface::new(
                ci.get_callback_interface_definition(self.name)
                    .context("CallbackInterface definition not found")?
                    .clone(),
                ci,
            ))?;
        declarations
            .initialization_code
            .insert(format!("{}Internals.register(lib)", self.canonical_name()));
        Ok(())
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceTemplate.kt")]
pub struct KotlinCallbackInterface {
    cbi: CallbackInterface,
}

impl KotlinCallbackInterface {
    pub fn new(cbi: CallbackInterface, _ci: &ComponentInterface) -> Self {
        Self { cbi }
    }

    // Functions used by the template code

    fn callback_internals(&self) -> String {
        internals_name(&self.cbi)
    }

    fn callback_interface_impl(&self) -> String {
        interface_impl_name(&self.cbi)
    }

    fn invoke_method_name(&self, meth: &Method) -> String {
        names::fn_name(&format!("invoke_{}", meth.name()))
    }
}

#[derive(Template, Hash)]
#[template(syntax = "kt", escape = "none", path = "CallbackInterfaceRuntime.kt")]
pub struct KotlinCallbackInterfaceRuntime;
