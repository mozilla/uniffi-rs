/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{Context, Result};
use heck::ToSnakeCase;
use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::{
    backend::{filters::to_rinja_error, TemplateExpression},
    interface::ir::*,
    interface::ComponentInterface,
};

mod visit_mut;

use visit_mut::{BindingsIrVisitor, Protocol, Runtimes};

// Config options to customize the generated python.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) cdylib_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomTypeConfig {
    type_name: Option<String>,
    imports: Option<Vec<String>>,
    into_custom: TemplateExpression,
    from_custom: TemplateExpression,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

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

// Generate python bindings for the given ComponentInterface, as a string.
pub fn generate_python_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    PythonBindingsIr::new(config.clone(), ci.clone())?
        .render()
        .context("failed to render python bindings")
}

/// Specializes the BindingsIr for Python
#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
struct PythonBindingsIr {
    imports: Vec<String>,
    ffi_definitions: Vec<FfiDefinition>,
    checksum_checks: Vec<ChecksumCheck>,
    type_definitions: Vec<TypeDefinition>,
    functions: Vec<Function>,
    module_docstring: Option<String>,
    globals: GlobalDefinitions,
    protocols: Vec<Protocol>,
    cdylib_name: String,
    runtimes: Runtimes,
    /// names to export via the __all__ object
    exports: Vec<String>,
}

impl PythonBindingsIr {
    fn new(config: Config, ci: ComponentInterface) -> Result<Self> {
        let mut ir: BindingsIr = ci.clone().try_into()?;
        let cdylib_name = config.cdylib_name();
        let mut visitor = BindingsIrVisitor::new(config);
        ir.visit_mut(&mut visitor)?;
        Ok(Self {
            imports: visitor.imports.into_iter().collect(),
            ffi_definitions: ir.ffi_definitions,
            checksum_checks: ir.checksum_checks,
            type_definitions: ir.type_definitions,
            functions: ir.functions,
            module_docstring: ir.crate_docstring,
            globals: ir.globals,
            protocols: visitor.protocols,
            cdylib_name,
            runtimes: visitor.runtimes,
            exports: visitor.exports,
        })
    }
}

#[derive(Clone, Default)]
pub struct PythonCodeOracle;

pub mod filters {

    use super::*;

    /// Fetch the `type_name` value set in `visit_mut.rs`
    pub fn type_name(node: impl AsType) -> rinja::Result<String> {
        node.as_type()
            .lang_data
            .get("type_name")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `type_name` {node:?}")))
    }

    /// Fetch the `ffi_converter_name` value set in `visit_mut.rs`
    pub fn ffi_converter_name(node: impl AsType) -> rinja::Result<String> {
        node.as_type()
            .lang_data
            .get("ffi_converter_name")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `ffi_converter_name` {node:?}")))
    }

    /// Fetch the `ffi_type_name` value set in `visit_mut.rs`
    pub fn ffi_type_name(node: impl AsFfiType) -> rinja::Result<String> {
        node.as_ffi_type()
            .lang_data
            .get("ffi_type_name")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `ffi_type_name` {node:?}")))
    }

    /// Fetch the `protocol_name` value set in `visit_mut.rs`
    pub fn protocol_name(node: &Interface) -> rinja::Result<String> {
        node.lang_data
            .get("protocol_name")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `protocol_name` {node:?}")))
    }

    /// Fetch the `literal` value set in `visit_mut.rs`
    pub fn literal(node: &Literal) -> rinja::Result<String> {
        node.lang_data
            .get("rendered_literal")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `rendered_literal` {node:?}")))
    }

    /// Fetch the `ffi_default_return` value set in `visit_mut.rs`
    pub fn ffi_default_return(node: &ReturnType) -> rinja::Result<String> {
        node.lang_data
            .get("ffi_default")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `ffi_default` {node:?}")))
    }

    /// Fetch the `base_classes` value set in `visit_mut.rs`
    pub fn base_classes(node: impl Node) -> rinja::Result<String> {
        node.lang_data()
            .get("base_classes")
            .ok_or_else(|| to_rinja_error(&format!("Error fetching `base_classes` {node:?}")))
    }

    /// Fetch the `custom_type_config` value set in `visit_mut.rs`
    pub fn custom_type_config(node: &CustomType) -> rinja::Result<Option<CustomTypeConfig>> {
        Ok(node.lang_data.get("custom_type_config"))
    }

    /// Fetch the `had_async_constructor` value set in `visit_mut.rs`
    pub fn had_async_constructor(node: &Interface) -> rinja::Result<bool> {
        Ok(node.lang_data.get("had_async_constructor").unwrap_or(false))
    }

    pub fn lower_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.lower", ffi_converter_name(node)?))
    }

    pub fn check_lower_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.check_lower", ffi_converter_name(node)?))
    }

    pub fn lift_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.lift", ffi_converter_name(node)?))
    }

    pub fn write_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.write", ffi_converter_name(node)?))
    }

    pub fn read_fn(node: impl AsType) -> rinja::Result<String> {
        Ok(format!("{}.read", ffi_converter_name(node)?))
    }

    pub fn return_type(callable: impl Callable) -> rinja::Result<String> {
        match &callable.return_type().ty {
            Some(ty) => type_name(ty),
            None => Ok("None".to_string()),
        }
    }

    /// Generate `def` or `async def` for a callable
    pub fn def(callable: impl Callable) -> rinja::Result<String> {
        if callable.is_async() {
            Ok("async def".to_string())
        } else {
            Ok("def".to_string())
        }
    }

    /// Generate a comma-separated list argument names and types
    pub fn arg_list(callable: impl Callable) -> rinja::Result<String> {
        let args = callable.arguments().iter().map(|a| {
            let ty = type_name(a)?;
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
    pub fn error_ffi_converter(callable: impl Callable) -> rinja::Result<String> {
        Ok(match &callable.throws_type().ty {
            Some(error_type) => {
                let ffi_converter_name = ffi_converter_name(error_type)?;
                match &error_type.kind {
                    TypeKind::Interface { .. } => format!("{ffi_converter_name}__as_error"),
                    _ => ffi_converter_name,
                }
            }
            None => "None".to_string(),
        })
    }

    pub fn ffi_return_type(node: impl FfiCallable) -> rinja::Result<String> {
        match &node.return_type().ty {
            Some(ty) => ffi_type_name(ty),
            None => Ok("None".to_string()),
        }
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
