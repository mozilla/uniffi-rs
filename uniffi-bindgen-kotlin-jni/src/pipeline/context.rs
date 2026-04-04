/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_bindgen::pipeline::general::sort::sort_type_definitions;

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    pub scaffolding_crate_name: Option<String>,
    pub current_crate_name: Option<String>,
    pub current_namespace_name: Option<String>,
    // Map namespace names to Rust crates
    pub crate_map: HashMap<String, String>,
    // Map namespace names to Config values
    pub config_map: HashMap<String, Config>,
    pub type_module_path_map: HashMap<Type, String>,
    pub package_map: HashMap<String, String>,
    pub type_id_map: HashMap<Type, u64>,
    pub ffi_type_oracle: FfiTypeOracle,
    pub layout_oracle: ffi_buffer::FfiBufferLayoutOracle,
    pub current_enum: Option<general::Enum>,
    pub types_used_as_error: HashSet<Type>,
}

impl Context {
    pub fn update_from_root(&mut self, root: &general::Root) -> Result<()> {
        self.populate_type_id_map(root);
        self.populate_type_module_path_map(root);
        self.populate_fields_from_type_definitions(root)?;
        self.populate_types_used_as_error(root);

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
        }
        Ok(())
    }

    fn populate_type_id_map(&mut self, root: &general::Root) {
        let mut counter = 0..;
        root.visit(|ty: &Type| {
            if !self.type_id_map.contains_key(ty) {
                self.type_id_map.insert(ty.clone(), counter.next().unwrap());
            }
        });
    }

    fn populate_types_used_as_error(&mut self, root: &general::Root) {
        root.visit(|type_node: &general::TypeNode| {
            if type_node.is_used_as_error && !self.types_used_as_error.contains(&type_node.ty) {
                self.types_used_as_error.insert(type_node.ty.clone());
            }
        });
    }

    fn populate_type_module_path_map(&mut self, root: &general::Root) {
        for namespace in root.namespaces.values() {
            for type_def in namespace.type_definitions.iter() {
                match type_def {
                    general::TypeDefinition::Record(rec) => {
                        self.type_module_path_map
                            .insert(rec.self_type.ty.clone(), rec.module_path.clone());
                    }
                    general::TypeDefinition::Enum(en) => {
                        self.type_module_path_map
                            .insert(en.self_type.ty.clone(), en.module_path.clone());
                    }
                    general::TypeDefinition::Interface(int) => {
                        self.type_module_path_map
                            .insert(int.self_type.ty.clone(), int.module_path.clone());
                    }
                    general::TypeDefinition::CallbackInterface(cbi) => {
                        self.type_module_path_map
                            .insert(cbi.self_type.ty.clone(), cbi.module_path.clone());
                    }
                    general::TypeDefinition::Custom(custom) => {
                        self.type_module_path_map
                            .insert(custom.self_type.ty.clone(), custom.module_path.clone());
                    }
                    _ => (),
                }
            }
        }
    }

    fn populate_fields_from_type_definitions(&mut self, root: &general::Root) -> Result<()> {
        use anyhow::Context;

        // Get type definitions for all packages and sort them.
        // This makes it so that dependencies come before their dependant types,
        // which simplifies the logic for the functions we're going to call.
        //
        // Note: recursive types can't be ordered in this manner and
        // the following functions should take that into account.
        let sorted_type_definitions = sort_type_definitions(
            root.namespaces
                .values()
                .flat_map(|n| n.type_definitions.iter().cloned()),
        );
        self.ffi_type_oracle
            .add_type_definitions(&sorted_type_definitions)
            .context("while building the type ffi oracle")?;
        self.layout_oracle
            .add_type_definitions(&sorted_type_definitions)
            .context("while building the ffi buffer layout oracle")?;
        Ok(())
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

    pub fn package_name_for_namespace(&self, namespace: &str) -> Result<&str> {
        self.package_map
            .get(namespace)
            .map(String::as_str)
            .ok_or_else(|| anyhow!("package name not found for namespace: {namespace:?}"))
    }

    pub fn rust_module_path_for_type(&self, ty: &Type) -> Result<String> {
        let module_path = self
            .type_module_path_map
            .get(ty)
            .ok_or_else(|| anyhow!("type module path not found: {ty:?}"))?;
        self.normalize_rust_module_path(module_path)
    }

    /// Normalize a Rust module path
    ///
    /// * Replace the current crate name with `crate`
    /// * Prefix other module paths with `::`
    pub fn normalize_rust_module_path(&self, module_path: &str) -> Result<String> {
        let Some(scaffolding_crate_name) = &self.scaffolding_crate_name else {
            // scaffolding_crate_name not set, probably because we're generating the bindings
            // No need to map anything
            return Ok(module_path.to_string());
        };
        if module_path == scaffolding_crate_name {
            Ok("crate".into())
        } else if let Some(rest) = module_path.strip_prefix(&format!("{scaffolding_crate_name}::"))
        {
            Ok(format!("crate::{rest}"))
        } else {
            Ok(format!("::{module_path}"))
        }
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

    pub fn crate_name(&self, namespace_name: &str) -> Result<&str> {
        self.crate_map
            .get(namespace_name)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("crate name not found: {namespace_name}"))
    }

    pub fn current_enum(&self) -> Result<&general::Enum> {
        self.current_enum
            .as_ref()
            .ok_or_else(|| anyhow!("current_enum not set"))
    }
}
