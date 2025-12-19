/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Organize the metadata, transforming it from a simple list to a more tree-like structure.

use anyhow::{anyhow, bail, Result};
use std::collections::{btree_map::Entry, BTreeMap};

use super::nodes::*;
use uniffi_pipeline::Node;

/// Converts `uniffi_meta` items into the initial IR.
///
/// Usage:
/// * Call [Self::add_metadata_item] with all `uniffi_meta` items in the interface (also [Self::add_module_docstring])
/// * Call [Self::try_into_initial_ir] to construct a `initial::Root` node.
#[derive(Default)]
pub struct UniffiMetaConverter {
    // Use BTreeMap for each of these so that things stay consistent, regardless of how the metadata is ordered.
    // There are 2 important names here we must not mix up. The namespace name and the rust crate name.
    // Our metadata usually carries "module_path" reflecting the crate name.
    // This maps module_paths to namespace names.
    module_path_map: BTreeMap<String, String>,
    // Everything below here keyed by namespace name, not the module_path.
    // Top level modules
    namespaces: BTreeMap<String, Namespace>,
    // Top-level type definitions and functions, keyed by module name
    module_docstrings: BTreeMap<String, String>,
    module_toml: BTreeMap<String, toml::Table>,
    functions: BTreeMap<String, BTreeMap<String, Function>>,
    records: BTreeMap<String, BTreeMap<String, Record>>,
    callback_interfaces: BTreeMap<String, BTreeMap<String, CallbackInterface>>,
    enums: BTreeMap<String, BTreeMap<String, Enum>>,
    custom_types: BTreeMap<String, BTreeMap<String, CustomType>>,
    interfaces: BTreeMap<String, BTreeMap<String, Interface>>,
    // Child items, keyed by module name + parent name
    constructors: BTreeMap<(String, String), BTreeMap<String, Constructor>>,
    methods: BTreeMap<(String, String), BTreeMap<String, Method>>,
    trait_methods: BTreeMap<(String, String), BTreeMap<String, TraitMethod>>,
    uniffi_traits: BTreeMap<(String, String), BTreeMap<String, UniffiTrait>>,
    trait_impls: BTreeMap<(String, String), BTreeMap<uniffi_meta::Type, ObjectTraitImpl>>,
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
                    .entry(module_path_to_namespace(&func.module_path))
                    .or_default()
                    .insert_unique(func.name.clone(), Function::try_from_node(func)?)?;
            }
            uniffi_meta::Metadata::Record(rec) => {
                self.records
                    .entry(module_path_to_namespace(&rec.module_path))
                    .or_default()
                    .insert_unique(rec.name.clone(), Record::try_from_node(rec)?)?;
            }
            uniffi_meta::Metadata::Enum(en) => {
                self.enums
                    .entry(module_path_to_namespace(&en.module_path))
                    .or_default()
                    .insert_unique(en.name.clone(), Enum::try_from_node(en)?)?;
            }
            uniffi_meta::Metadata::Object(int) => {
                self.interfaces
                    .entry(module_path_to_namespace(&int.module_path))
                    .or_default()
                    .insert_unique(int.name.clone(), Interface::try_from_node(int)?)?;
            }
            uniffi_meta::Metadata::CallbackInterface(cbi) => {
                self.callback_interfaces
                    .entry(module_path_to_namespace(&cbi.module_path))
                    .or_default()
                    .insert_unique(cbi.name.clone(), CallbackInterface::try_from_node(cbi)?)?;
            }
            uniffi_meta::Metadata::CustomType(custom) => {
                self.custom_types
                    .entry(module_path_to_namespace(&custom.module_path))
                    .or_default()
                    .insert_unique(custom.name.clone(), CustomType::try_from_node(custom)?)?;
            }
            uniffi_meta::Metadata::Constructor(cons) => {
                self.constructors
                    .entry((
                        module_path_to_namespace(&cons.module_path),
                        cons.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(cons.name.clone(), Constructor::try_from_node(cons)?)?;
            }
            uniffi_meta::Metadata::Method(meth) => {
                self.methods
                    .entry((
                        module_path_to_namespace(&meth.module_path),
                        meth.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(meth.name.clone(), Method::try_from_node(meth)?)?;
            }
            uniffi_meta::Metadata::TraitMethod(meth) => {
                self.trait_methods
                    .entry((
                        module_path_to_namespace(&meth.module_path),
                        meth.trait_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(meth.name.clone(), TraitMethod::try_from_node(meth)?)?;
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
                        module_path_to_namespace(&meth.module_path),
                        meth.self_name.to_string(),
                    ))
                    .or_default()
                    .insert_unique(ut.name().to_string(), UniffiTrait::try_from_node(ut)?)?;
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
                    .entry((module_path_to_namespace(module_path), name.to_string()))
                    .or_default()
                    .insert_unique(imp.trait_ty.clone(), ObjectTraitImpl::try_from_node(imp)?)?;
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

    pub fn try_into_initial_ir(mut self) -> Result<Root> {
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
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .functions
                .extend(funcs.into_values());
        }
        for (module_path, list) in self.records {
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(
                    list.into_values()
                        .map(|mut r| {
                            let key = (module_path.clone(), r.name.clone());
                            if let Some(methods) = self.methods.remove(&key) {
                                r.methods.extend(methods.into_values());
                            }
                            if let Some(constructors) = self.constructors.remove(&key) {
                                r.constructors.extend(constructors.into_values())
                            }
                            if let Some(uniffi_traits) = self.uniffi_traits.remove(&key) {
                                r.uniffi_traits.extend(uniffi_traits.into_values())
                            }
                            Ok(TypeDefinition::Record(r))
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
        }
        for (module_path, list) in self.enums {
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(
                    list.into_values()
                        .map(|mut e| {
                            let key = (module_path.clone(), e.name.clone());
                            if let Some(methods) = self.methods.remove(&key) {
                                e.methods.extend(methods.into_values());
                            }
                            if let Some(constructors) = self.constructors.remove(&key) {
                                e.constructors.extend(constructors.into_values())
                            }
                            if let Some(uniffi_traits) = self.uniffi_traits.remove(&key) {
                                e.uniffi_traits.extend(uniffi_traits.into_values())
                            }
                            Ok(TypeDefinition::Enum(e))
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
        }
        for (module_path, list) in self.custom_types {
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(list.into_values().map(TypeDefinition::Custom));
        }
        // Collect child items for interfaces and callback interfaces
        for (module_path, list) in self.interfaces {
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(
                    list.into_values()
                        .map(|mut int| {
                            let key = (module_path.clone(), int.name.clone());
                            if let Some(methods) = self.methods.remove(&key) {
                                if self.trait_methods.contains_key(&key) {
                                    // Trait methods have an explicit index, so mixing them with
                                    // regular methods won't work.
                                    bail!("{} contains both methods and trait methods", int.name)
                                }
                                int.methods.extend(methods.into_values());
                            } else if let Some(trait_methods) = self.trait_methods.remove(&key) {
                                int.methods.extend(Self::convert_trait_methods(
                                    trait_methods.into_values().collect(),
                                ));
                            }
                            if let Some(constructors) = self.constructors.remove(&key) {
                                int.constructors.extend(constructors.into_values())
                            }
                            if let Some(uniffi_traits) = self.uniffi_traits.remove(&key) {
                                int.uniffi_traits.extend(uniffi_traits.into_values())
                            }
                            if let Some(trait_impls) = self.trait_impls.remove(&key) {
                                int.trait_impls.extend(trait_impls.into_values())
                            }
                            Ok(TypeDefinition::Interface(int))
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
        }
        for (module_path, list) in self.callback_interfaces {
            get_namespace(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(list.into_values().map(|mut cbi| {
                    let key = (module_path.clone(), cbi.name.clone());
                    if let Some(trait_methods) = self.trait_methods.remove(&key) {
                        cbi.methods.extend(Self::convert_trait_methods(
                            trait_methods.into_values().collect(),
                        ));
                    }
                    TypeDefinition::CallbackInterface(cbi)
                }))
        }
        if !self.constructors.is_empty() {
            bail!("Leftover constructors: {:?}", self.constructors)
        }
        if !self.methods.is_empty() {
            bail!("Leftover methods: {:?}", self.methods)
        }
        if !self.trait_methods.is_empty() {
            bail!("Leftover trait_methods: {:?}", self.trait_methods)
        }
        if !self.uniffi_traits.is_empty() {
            bail!("Leftover uniffi_traits: {:?}", self.uniffi_traits)
        }
        if !self.trait_impls.is_empty() {
            bail!("Leftover trait_impls: {:?}", self.trait_impls)
        }
        // set the namespace names
        root.try_visit_mut(|ty: &mut Type| match ty {
            Type::Interface {
                module_path,
                namespace,
                ..
            }
            | Type::Record {
                module_path,
                namespace,
                ..
            }
            | Type::Enum {
                module_path,
                namespace,
                ..
            }
            | Type::CallbackInterface {
                module_path,
                namespace,
                ..
            }
            | Type::Custom {
                module_path,
                namespace,
                ..
            } => {
                *namespace = get_namespace_name(&self.module_path_map, module_path)?.to_string();
                Ok(())
            }
            _ => Ok(()),
        })?;
        Ok(root)
    }

    fn convert_trait_methods(mut trait_methods: Vec<TraitMethod>) -> impl Iterator<Item = Method> {
        trait_methods.sort_by_key(|tm| tm.index);
        trait_methods.into_iter().map(Self::convert_trait_method)
    }

    fn convert_trait_method(trait_method: TraitMethod) -> Method {
        Method {
            name: trait_method.name,
            is_async: trait_method.is_async,
            inputs: trait_method.inputs,
            return_type: trait_method.return_type,
            throws: trait_method.throws,
            checksum: trait_method.checksum,
            docstring: trait_method.docstring,
        }
    }
}

fn module_path_to_namespace(module_path: &str) -> String {
    module_path.split("::").next().unwrap().to_string()
}

fn get_namespace_name<'a>(
    module_path_map: &'a BTreeMap<String, String>,
    module_path: &str,
) -> Result<&'a str> {
    let crate_name = module_path.split("::").next().unwrap();
    module_path_map
        .get(crate_name)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("module lookup failed: {module_path:?}"))
}

fn get_namespace<'a>(
    module_path_map: &BTreeMap<String, String>,
    root: &'a mut Root,
    module_path: &str,
) -> Result<&'a mut Namespace> {
    let name = get_namespace_name(module_path_map, module_path)?;
    root.namespaces
        .get_mut(name)
        .ok_or_else(|| anyhow!("root module lookup failed: {module_path:?}"))
}
