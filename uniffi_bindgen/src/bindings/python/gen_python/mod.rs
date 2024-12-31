/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use heck::ToSnakeCase;
use rinja::Template;

use crate::backend::TemplateExpression;
use crate::interface::{ir::BindingsIr, ComponentInterface};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;

mod convert_ir;
mod dataclasses;
mod ffi;
mod functions;
mod interfaces;
mod types;

pub use convert_ir::*;
pub use dataclasses::*;
pub use ffi::*;
pub use functions::*;
pub use interfaces::*;
pub use types::*;

// Generate python bindings for the given ComponentInterface, as a string.
pub fn generate_python_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    PythonBindingsIr::from_ci(ci.clone(), config.clone())?
        .render()
        .context("failed to render python bindings")
}

pub fn generate_python_bindings_from_ir(ir: PythonBindingsIr) -> Result<String> {
    ir.render().context("failed to render python bindings")
}

// Config options to customize the generated python.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) cdylib_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
}

impl Config {
    /// Get the package name for a given external namespace.
    pub fn module_for_namespace(&self, ns: &str) -> String {
        let ns = ns.to_string().to_snake_case();
        match self.external_packages.get(&ns) {
            None => format!(".{ns}"),
            Some(value) if value.is_empty() => ns,
            Some(value) => format!("{value}.{ns}"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomTypeConfig {
    type_name: Option<String>,
    imports: Option<Vec<String>>,
    into_custom: TemplateExpression,
    from_custom: TemplateExpression,
}

/// Python Bindings Ir
///
/// This is constructed from the general `BindingsIr` -- specializing it for Python.
/// It's then used as a Rinja template to generate the Python source
#[derive(Debug, Clone, rinja::Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
pub struct PythonBindingsIr {
    pub namespace: String,
    pub cdylib_name: String,
    pub module_docstring: Option<String>,
    pub runtimes: Runtimes,
    pub globals: GlobalNodes,
    pub ffi_definitions: Vec<FfiDefinition>,
    pub protocols: Vec<Protocol>,
    pub type_definitions: Vec<TypeDefinition>,
    pub functions: Vec<Function>,
    pub checksum_checks: Vec<ChecksumCheck>,
    /// Import statements, using a BTreeset to sort/de-dupe them
    pub imports: BTreeSet<String>,
    /// Names to export via the `__all__` attribute
    pub exports: Vec<String>,
}

impl PythonBindingsIr {
    pub fn from_ci(ci: ComponentInterface, config: Config) -> Result<Self> {
        Self::from_general_ir(ci.try_into()?, config)
    }

    pub fn from_general_ir(ir: BindingsIr, config: Config) -> Result<Self> {
        convert_bindings_ir(ir, config.clone())
    }
}

/// Global definitions
///
/// These are nodes that we always define for the bindings.
/// Putting them here means that we always have easy access them from the bindings generation code.
#[derive(Debug, Clone)]
pub struct GlobalNodes {
    pub ffi_rustbuffer_alloc: String,
    pub ffi_rustbuffer_reserve: String,
    pub ffi_rustbuffer_free: String,
    /// FFI function to check the contract version
    pub ffi_uniffi_contract_version: String,
    /// FFI function type for freeing a callback interface
    pub callback_interface_free_type: String,
    /// Always defined String, since it's used to handle Rust call errors
    pub string_type: Type,
    pub contract_version: u32,
}

// These are sections of helper code that we load once
#[derive(Default, Debug, Clone)]
pub struct Runtimes {
    pub async_: bool,
    pub async_callback: bool,
    pub callback_interface: bool,
}

/// Protocol to define
#[derive(Debug, Clone)]
pub struct Protocol {
    pub name: String,
    pub base_class: String,
    pub docstring: Option<String>,
    pub methods: Vec<Method>,
}

pub mod filters {

    use super::*;

    pub fn lower_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.lower", node.as_type().ffi_converter_name))
    }

    pub fn check_lower_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.check_lower", node.as_type().ffi_converter_name))
    }

    pub fn lift_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.lift", node.as_type().ffi_converter_name))
    }

    pub fn write_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.write", node.as_type().ffi_converter_name))
    }

    pub fn read_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.read", node.as_type().ffi_converter_name))
    }

    pub fn return_type(callable: impl AsCallable) -> rinja::Result<String> {
        Ok(match callable.return_type() {
            Some(ty) => ty.type_name.clone(),
            None => "None".to_string(),
        })
    }

    /// Generate `def` or `async def` for a callable
    pub fn def(callable: impl AsCallable) -> rinja::Result<String> {
        if callable.is_async() {
            Ok("async def".to_string())
        } else {
            Ok("def".to_string())
        }
    }

    /// Generate a comma-separated list argument names and types
    pub fn arg_list(callable: impl AsCallable) -> rinja::Result<String> {
        let args = callable.arguments().iter().map(|a| {
            let ty = &a.ty.type_name;
            let name = &a.name;
            Ok(match &a.default {
                Some(_) => format!("{name}: typing.Union[object, {ty}] = _DEFAULT"),
                None => format!("{name}: {ty}"),
            })
        });
        let self_arg = (callable.is_method() || callable.is_primary_constructor())
            .then(|| Ok("self".to_string()));

        Ok(self_arg
            .into_iter()
            .chain(args)
            .collect::<rinja::Result<Vec<_>>>()?
            .join(", "))
    }

    /// Get the FFI converter for a throws type
    ///
    /// Returns "None" if there isn't a throws type.
    pub fn error_ffi_converter(callable: impl AsCallable) -> rinja::Result<String> {
        Ok(match callable.throws_type() {
            Some(error_type) => {
                let ffi_converter_name = &error_type.ffi_converter_name;
                match &error_type.kind {
                    uniffi_meta::Type::Object { .. } => format!("{ffi_converter_name}__as_error"),
                    _ => ffi_converter_name.clone(),
                }
            }
            None => "None".to_string(),
        })
    }

    /// Indent a docstring
    ///
    /// For Some values will indent each line, except the first by `spaces`.  
    /// For None, it will return the empty string.
    ///
    /// This gets both cases right with template code that looks like this:
    ///
    /// ```python
    /// {{ meth.docstring|docindent(4) }}
    /// fn {{ meth.name }}(...)
    /// ```
    pub fn docindent(docstring: &Option<String>, spaces: usize) -> rinja::Result<String> {
        Ok(match docstring {
            None => "".to_string(),
            Some(docstring) => {
                let mut output = String::new();
                let leading_space = " ".repeat(spaces);
                for line in docstring.split('\n') {
                    output.push_str(line);
                    output.push('\n');
                    output.push_str(&leading_space);
                }
                output
            }
        })
    }
}
