/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    // Name of the current package when we're generating scaffolding
    pub scaffolding_crate_name: Option<String>,
    // Map namespace names to Kotlin packages
    pub package_map: HashMap<String, String>,
    // Map namespace names to Rust crates
    pub crate_map: HashMap<String, String>,
    // Map namespace names to Config values
    pub config_map: HashMap<String, Config>,
    // Map namespace / type names the type definition module_path
    pub type_module_paths: HashMap<String, HashMap<String, String>>,
    // Map namespace / function names the module_path
    pub function_module_paths: HashMap<String, HashMap<String, String>>,
    pub current_crate_name: Option<String>,
    pub current_namespace_name: Option<String>,
    pub current_enum: Option<general::Enum>,
    pub types_used_as_error: HashSet<Type>,
}

impl Context {
    pub fn update_from_root(&mut self, root: &general::Root) -> Result<()> {
        for namespace in root.namespaces.values() {
            let config = Config::from_toml(namespace.config_toml.as_deref())?;
            let package_name = match &config.package_name {
                Some(name) => name.clone(),
                None => format!("uniffi.{}", namespace.name),
            };
            self.config_map.insert(namespace.name.clone(), config);
            self.package_map
                .insert(namespace.name.clone(), package_name);
            self.crate_map
                .insert(namespace.name.clone(), namespace.crate_name.clone());
            self.type_module_paths
                .insert(namespace.name.clone(), Self::type_module_paths(namespace));
            self.function_module_paths.insert(
                namespace.name.clone(),
                Self::function_module_paths(namespace),
            );
        }
        root.visit(|type_node: &general::TypeNode| {
            if type_node.is_used_as_error && !self.types_used_as_error.contains(&type_node.ty) {
                self.types_used_as_error.insert(type_node.ty.clone());
            }
        });
        Ok(())
    }

    fn type_module_paths(namespace: &general::Namespace) -> HashMap<String, String> {
        namespace
            .type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                general::TypeDefinition::Record(rec) => {
                    Some((rec.orig_name.clone(), rec.module_path.clone()))
                }
                general::TypeDefinition::Enum(en) => {
                    Some((en.orig_name.clone(), en.module_path.clone()))
                }
                general::TypeDefinition::Interface(int) => {
                    Some((int.orig_name.clone(), int.module_path.clone()))
                }
                general::TypeDefinition::CallbackInterface(cbi) => {
                    Some((cbi.orig_name.clone(), cbi.module_path.clone()))
                }
                general::TypeDefinition::Custom(custom) => {
                    Some((custom.orig_name.clone(), custom.module_path.clone()))
                }
                _ => None,
            })
            .collect()
    }

    fn function_module_paths(namespace: &general::Namespace) -> HashMap<String, String> {
        namespace
            .functions
            .iter()
            .map(|f| (f.callable.orig_name.clone(), f.module_path.clone()))
            .collect()
    }

    pub fn update_from_namespace(&mut self, namespace: &general::Namespace) {
        self.current_crate_name = Some(namespace.crate_name.clone());
        self.current_namespace_name = Some(namespace.name.clone());
    }

    pub fn update_from_enum(&mut self, en: &general::Enum) {
        self.current_enum = Some(en.clone());
    }

    pub fn current_crate_name(&self) -> Result<&str> {
        self.current_crate_name
            .as_deref()
            .ok_or_else(|| anyhow!("current_crate_name not set"))
    }

    pub fn namespace_name(&self) -> Result<&str> {
        self.current_namespace_name
            .as_deref()
            .ok_or_else(|| anyhow!("current_namespace_name not set"))
    }

    pub fn config(&self) -> Result<&Config> {
        let namespace_name = self.namespace_name()?;
        self.config_map
            .get(namespace_name)
            .ok_or_else(|| anyhow!("config not found: {namespace_name}"))
    }

    pub fn package_name(&self, namespace_name: &str) -> Result<&str> {
        self.package_map
            .get(namespace_name)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("package name not found: {namespace_name}"))
    }

    pub fn current_package_name(&self) -> Result<&str> {
        self.package_name(self.namespace_name()?)
    }

    pub fn crate_name(&self, namespace_name: &str) -> Result<&str> {
        self.crate_map
            .get(namespace_name)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("crate name not found: {namespace_name}"))
    }

    pub fn rust_module_path_for_func(
        &self,
        namespace: &str,
        func_orig_name: &str,
    ) -> Result<String> {
        let module_path = self
            .function_module_paths
            .get(namespace)
            .and_then(|map| map.get(func_orig_name))
            .map(|s| s.as_str())
            .ok_or_else(|| {
                anyhow!("function module path not found: {namespace}:{func_orig_name}")
            })?;
        self.normalize_rust_module_path(module_path)
    }

    pub fn rust_module_path_for_type(
        &self,
        namespace: &str,
        type_orig_name: &str,
    ) -> Result<String> {
        let module_path = self
            .type_module_paths
            .get(namespace)
            .and_then(|map| map.get(type_orig_name))
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("type module path not found: {namespace}:{type_orig_name}"))?;
        self.normalize_rust_module_path(module_path)
    }

    /// Normalize a Rust module path
    ///
    /// This replaces the current module name with `crate`
    pub fn normalize_rust_module_path(&self, module_path: &str) -> Result<String> {
        let Some(scaffolding_package_name) = &self.scaffolding_crate_name else {
            // scaffolding_package_name not set, probably because we're generating the bindings
            // No need to map anything
            return Ok(module_path.to_string());
        };
        if module_path == scaffolding_package_name {
            Ok("crate".into())
        } else if let Some(rest) =
            module_path.strip_prefix(&format!("{scaffolding_package_name}::"))
        {
            Ok(format!("crate::{rest}"))
        } else {
            Ok(format!("::{module_path}"))
        }
    }

    pub fn current_enum(&self) -> Result<&general::Enum> {
        self.current_enum
            .as_ref()
            .ok_or_else(|| anyhow!("current_enum not set"))
    }
}
