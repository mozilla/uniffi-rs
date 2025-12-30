/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;

use super::*;

pub struct Context {
    // Map crate names to namespace names
    //
    // Depending on how we learn about metadata items, we may know the crate name, full module
    // path, or namespace name.  Items from UDL usually have the namespace name, while items from
    // the proc-macro usually have the full module path.  This map allows us to normalize
    // everything to use the namespace.
    pub module_path_map: BTreeMap<String, String>,
    // Child items for interfaces/records etc.
    pub constructors:
        BTreeMap<(String, String), BTreeMap<String, uniffi_meta::ConstructorMetadata>>,
    pub methods: BTreeMap<(String, String), BTreeMap<String, uniffi_meta::MethodMetadata>>,
    pub trait_methods:
        BTreeMap<(String, String), BTreeMap<String, uniffi_meta::TraitMethodMetadata>>,
    pub uniffi_traits:
        BTreeMap<(String, String), BTreeMap<String, uniffi_meta::UniffiTraitMetadata>>,
    pub trait_impls: BTreeMap<
        (String, String),
        BTreeMap<uniffi_meta::Type, uniffi_meta::ObjectTraitImplMetadata>,
    >,
}

impl Context {
    pub fn get_namespace_name(&self, module_path: &str) -> Result<String> {
        let crate_name = module_path.split("::").next().unwrap();
        self.module_path_map
            .get(crate_name)
            .cloned()
            .ok_or_else(|| anyhow!("module lookup failed: {module_path:?}"))
    }

    pub fn methods_for_type(&self, module_path: &str, type_name: &str) -> Result<Vec<Method>> {
        let crate_name = module_path.split("::").next().unwrap();
        let child_key = (crate_name.to_string(), type_name.to_string());

        if let Some(methods) = self.methods.get(&child_key) {
            if self.trait_methods.contains_key(&child_key) {
                // Trait methods have an explicit index, so mixing them with
                // regular methods won't work.
                bail!("{} contains both methods and trait methods", type_name)
            }
            methods
                .values()
                .cloned()
                .map(|meth| meth.map_node(self))
                .collect()
        } else if let Some(trait_methods) = self.trait_methods.get(&child_key) {
            let mut methods = Vec::from_iter(trait_methods.values());
            methods.sort_by_key(|tm| tm.index);
            methods
                .iter()
                .map(|meth| (*meth).clone().map_node(self))
                .collect()
        } else {
            Ok(vec![])
        }
    }

    pub fn constructors_for_type(
        &self,
        module_path: &str,
        type_name: &str,
    ) -> Result<Vec<Constructor>> {
        let crate_name = module_path.split("::").next().unwrap();
        let child_key = (crate_name.to_string(), type_name.to_string());

        if let Some(constructors) = self.constructors.get(&child_key) {
            constructors
                .values()
                .cloned()
                .map(|cons| cons.map_node(self))
                .collect()
        } else {
            Ok(vec![])
        }
    }

    pub fn uniffi_traits_for_type(
        &self,
        module_path: &str,
        type_name: &str,
    ) -> Result<Vec<UniffiTrait>> {
        let crate_name = module_path.split("::").next().unwrap();
        let child_key = (crate_name.to_string(), type_name.to_string());

        if let Some(uniffi_trait) = self.uniffi_traits.get(&child_key) {
            uniffi_trait
                .values()
                .cloned()
                .map(|ut| ut.map_node(self))
                .collect()
        } else {
            Ok(vec![])
        }
    }

    pub fn trait_impls_for_type(
        &self,
        module_path: &str,
        type_name: &str,
    ) -> Result<Vec<ObjectTraitImpl>> {
        let crate_name = module_path.split("::").next().unwrap();
        let child_key = (crate_name.to_string(), type_name.to_string());

        if let Some(trait_impl) = self.trait_impls.get(&child_key) {
            trait_impl
                .values()
                .cloned()
                .map(|trait_impl| trait_impl.map_node(self))
                .collect()
        } else {
            Ok(vec![])
        }
    }
}
