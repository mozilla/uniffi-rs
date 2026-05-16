/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    pub scaffolding_crate_name: Option<String>,
    pub current_crate_name: Option<String>,
    pub type_module_path_map: HashMap<Type, String>,
    pub package_map: HashMap<String, String>,
}

impl Context {
    pub fn update_from_root(&mut self, root: &general::Root) -> Result<()> {
        self.populate_type_module_path_map(root);
        for namespace in root.namespaces.values() {
            let package_name = format!("uniffi.{}", namespace.name);
            self.package_map
                .insert(namespace.name.clone(), package_name);
        }
        Ok(())
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

    pub fn update_from_namespace(&mut self, namespace: &general::Namespace) {
        self.current_crate_name = Some(namespace.crate_name.clone());
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
}
