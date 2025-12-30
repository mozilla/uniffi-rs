/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Organize the metadata, transforming it from a simple list to a more tree-like structure.

use anyhow::{anyhow, bail, Result};
use std::collections::{btree_map::Entry, BTreeMap};

use super::*;

/// Converts `uniffi_meta` items into the initial IR.
///
/// Usage:
/// * Call [Self::add_metadata_item] with all `uniffi_meta` items in the interface (also [Self::add_module_docstring])
/// * Call [Self::try_into_initial_ir] to construct a `initial::Root` node.
#[derive(Default)]
pub struct UniffiMetaConverter {
    module_path_map: BTreeMap<String, String>,
    // Everything below here keyed by namespace name, not the module_path.
    // Top level modules
    namespaces: BTreeMap<String, Namespace>,
    // Top-level type definitions and functions, keyed by module name
    module_docstrings: BTreeMap<String, String>,
    module_toml: BTreeMap<String, toml::Table>,
    functions: BTreeMap<String, BTreeMap<String, uniffi_meta::FnMetadata>>,
    records: BTreeMap<String, BTreeMap<String, uniffi_meta::RecordMetadata>>,
    callback_interfaces: BTreeMap<String, BTreeMap<String, uniffi_meta::CallbackInterfaceMetadata>>,
    enums: BTreeMap<String, BTreeMap<String, uniffi_meta::EnumMetadata>>,
    custom_types: BTreeMap<String, BTreeMap<String, uniffi_meta::CustomTypeMetadata>>,
    interfaces: BTreeMap<String, BTreeMap<String, uniffi_meta::ObjectMetadata>>,
    // Child items, keyed by module name + parent name
    constructors: BTreeMap<(String, String), BTreeMap<String, uniffi_meta::ConstructorMetadata>>,
    methods: BTreeMap<(String, String), BTreeMap<String, uniffi_meta::MethodMetadata>>,
    trait_methods: BTreeMap<(String, String), BTreeMap<String, uniffi_meta::TraitMethodMetadata>>,
    uniffi_traits: BTreeMap<(String, String), BTreeMap<String, uniffi_meta::UniffiTraitMetadata>>,
    trait_impls: BTreeMap<
        (String, String),
        BTreeMap<uniffi_meta::Type, uniffi_meta::ObjectTraitImplMetadata>,
    >,
}

/// Utility trait used to insert metadata items into a BTreeMap, but bail on duplicates
trait InsertUnique<K, V> {
    fn insert_unique(&mut self, k: K, v: V) -> Result<()>;
}

impl<K, V> InsertUnique<K, V> for BTreeMap<K, V>
where
    K: std::fmt::Debug + Ord,
    V: std::fmt::Debug + PartialEq,
{
    fn insert_unique(&mut self, k: K, v: V) -> Result<()> {
        match self.entry(k) {
            Entry::Vacant(e) => {
                e.insert(v);
                Ok(())
            }
            Entry::Occupied(e) => {
                if e.get() != &v {
                    bail!(
                        "Conflicting metadata types:\nold: {:?}\nnew: {v:?}",
                        e.get()
                    );
                }
                Ok(())
            }
        }
    }
}

impl UniffiMetaConverter {
    /// Add a [uniffi_meta::Metadata] item to be converted
    pub fn add_metadata_item(&mut self, meta: uniffi_meta::Metadata) -> Result<()> {
        match meta {
            uniffi_meta::Metadata::Namespace(namespace) => {
                self.module_path_map
                    .insert(namespace.crate_name.clone(), namespace.name.clone());
                // Insert a new module
                self.namespaces.insert_unique(
                    namespace.name.clone(),
                    Namespace {
                        crate_name: namespace.crate_name,
                        docstring: None,
                        config_toml: None,
                        name: namespace.name,
                        functions: vec![],
                        type_definitions: vec![],
                    },
                )?;
            }
            uniffi_meta::Metadata::Func(func) => {
                self.functions
                    .entry(module_path_to_crate_name(&func.module_path))
                    .or_default()
                    .insert_unique(func.name.clone(), func)?;
            }
            uniffi_meta::Metadata::Record(rec) => {
                self.records
                    .entry(module_path_to_crate_name(&rec.module_path))
                    .or_default()
                    .insert_unique(rec.name.clone(), rec)?;
            }
            uniffi_meta::Metadata::Enum(en) => {
                self.enums
                    .entry(module_path_to_crate_name(&en.module_path))
                    .or_default()
                    .insert_unique(en.name.clone(), en)?;
            }
            uniffi_meta::Metadata::Object(int) => {
                self.interfaces
                    .entry(module_path_to_crate_name(&int.module_path))
                    .or_default()
                    .insert_unique(int.name.clone(), int)?;
            }
            uniffi_meta::Metadata::CallbackInterface(cbi) => {
                self.callback_interfaces
                    .entry(module_path_to_crate_name(&cbi.module_path))
                    .or_default()
                    .insert_unique(cbi.name.clone(), cbi)?;
            }
            uniffi_meta::Metadata::CustomType(custom) => {
                self.custom_types
                    .entry(module_path_to_crate_name(&custom.module_path))
                    .or_default()
                    .insert_unique(custom.name.clone(), custom)?;
            }
            uniffi_meta::Metadata::Constructor(cons) => {
                self.constructors
                    .entry((
                        module_path_to_crate_name(&cons.module_path),
                        cons.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(cons.name.clone(), cons)?;
            }
            uniffi_meta::Metadata::Method(meth) => {
                self.methods
                    .entry((
                        module_path_to_crate_name(&meth.module_path),
                        meth.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(meth.name.clone(), meth)?;
            }
            uniffi_meta::Metadata::TraitMethod(meth) => {
                self.trait_methods
                    .entry((
                        module_path_to_crate_name(&meth.module_path),
                        meth.trait_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(meth.name.clone(), meth)?;
            }
            uniffi_meta::Metadata::UniffiTrait(ut) => {
                let meth = match &ut {
                    uniffi_meta::UniffiTraitMetadata::Debug { fmt } => fmt,
                    uniffi_meta::UniffiTraitMetadata::Display { fmt } => fmt,
                    uniffi_meta::UniffiTraitMetadata::Eq { eq, .. } => eq,
                    uniffi_meta::UniffiTraitMetadata::Hash { hash } => hash,
                    uniffi_meta::UniffiTraitMetadata::Ord { cmp } => cmp,
                };

                self.uniffi_traits
                    .entry((
                        module_path_to_crate_name(&meth.module_path),
                        meth.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(ut.name().to_string(), ut)?;
            }
            uniffi_meta::Metadata::ObjectTraitImpl(imp) => {
                let (module_path, name) = match &imp.ty {
                    uniffi_meta::Type::Object {
                        module_path, name, ..
                    }
                    | uniffi_meta::Type::Record {
                        module_path, name, ..
                    }
                    | uniffi_meta::Type::Enum {
                        module_path, name, ..
                    }
                    | uniffi_meta::Type::Custom {
                        module_path, name, ..
                    }
                    | uniffi_meta::Type::CallbackInterface {
                        module_path, name, ..
                    } => (module_path, name),
                    _ => bail!("Invalid ObjectTraitImpl type: {:?}", imp.ty),
                };
                self.trait_impls
                    .entry((module_path_to_crate_name(module_path), name.to_string()))
                    .or_default()
                    .insert_unique(imp.trait_ty.clone(), imp)?;
            }
            uniffi_meta::Metadata::UdlFile(_) => (),
        }
        Ok(())
    }

    pub fn add_module_config_toml(
        &mut self,
        module_name: String,
        table: toml::Table,
    ) -> Result<()> {
        self.module_toml.insert_unique(module_name, table)?;
        Ok(())
    }

    /// Add a docstring for a module,
    ///
    /// This is currently UDL-specific.  Eventually, we should probably make this another metadata
    /// items
    pub fn add_module_docstring(&mut self, namespace: String, docstring: String) -> Result<()> {
        self.module_docstrings.insert_unique(namespace, docstring)
    }

    pub fn try_into_initial_ir(self) -> Result<Root> {
        let context = Context {
            module_path_map: self.module_path_map.clone(),
            constructors: self.constructors,
            methods: self.methods,
            trait_methods: self.trait_methods,
            uniffi_traits: self.uniffi_traits,
            trait_impls: self.trait_impls,
        };

        let mut root = Root {
            namespaces: self.namespaces.into_iter().collect(),
            cdylib: None,
        };

        // Move child items into their parents
        for (namespace_name, docstring) in self.module_docstrings {
            // already the namespace name, so no need to convert.
            let namespace = root.namespaces.get_mut(&namespace_name).ok_or_else(|| {
                anyhow!("namespace specified in toml doesn't exist: {namespace_name:?}")
            })?;
            namespace.docstring = Some(docstring);
        }
        for (namespace_name, table) in self.module_toml {
            // already the namespace name, so no need to convert.
            // we should maybe ignore an error here?
            let namespace = root.namespaces.get_mut(&namespace_name).ok_or_else(|| {
                anyhow!("namespace specified in toml doesn't exist: {namespace_name:?}")
            })?;
            // ideally `namespace.config_toml` would be a `toml::Table`, but all members must implement `Node`.
            namespace.config_toml = Some(toml::to_string(&table)?);
        }
        for (module_path, funcs) in self.functions {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for func in funcs.into_values() {
                namespace.functions.push(func.map_node(&context)?);
            }
        }
        for (module_path, list) in self.records {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for rec in list.into_values() {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Record(rec.map_node(&context)?));
            }
        }
        for (module_path, list) in self.enums {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for en in list.into_values() {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Enum(en.map_node(&context)?));
            }
        }
        for (module_path, list) in self.custom_types {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for custom in list.into_values() {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Custom(custom.map_node(&context)?));
            }
        }
        // Collect child items for interfaces and callback interfaces
        for (module_path, list) in self.interfaces {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for int in list.into_values() {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Interface(int.map_node(&context)?));
            }
        }
        for (module_path, list) in self.callback_interfaces {
            let namespace = get_namespace(&self.module_path_map, &mut root, &module_path)?;
            for cbi in list.into_values() {
                namespace
                    .type_definitions
                    .push(TypeDefinition::CallbackInterface(cbi.map_node(&context)?));
            }
        }
        Ok(root)
    }
}

fn module_path_to_crate_name(module_path: &str) -> String {
    module_path.split("::").next().unwrap().to_string()
}

fn get_namespace<'a>(
    module_path_map: &BTreeMap<String, String>,
    root: &'a mut Root,
    module_path: &str,
) -> Result<&'a mut Namespace> {
    let crate_name = module_path.split("::").next().unwrap();
    let namespace_name = module_path_map
        .get(crate_name)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("module lookup failed: {module_path:?}"))?;
    root.namespaces
        .get_mut(namespace_name)
        .ok_or_else(|| anyhow!("root module lookup failed: {module_path:?}"))
}
