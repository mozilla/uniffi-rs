/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Organize the metadata, transforming it from a simple list to a more tree-like structure.

use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8Path;
use std::{collections::BTreeMap, fs};

use super::nodes::*;
use uniffi_pipeline::Node;

/// Converts `uniffi_meta` items into the initial IR.
///
/// Usage:
/// * Call [Self::add_metadata_item] with all `uniffi_meta` items in the interface (also [Self::add_module_docstring])
/// * Call [Self::try_into_initial_ir] to construct a `initial::Root` node.
#[derive(Default)]
pub struct UniffiMetaConverter {
    // Map module_path to `Module::name`
    //
    // This is needed because UDL and proc-macro code can generate different module paths.  To
    // normalize things, we find the `NamespaceMetadata::name` that matches the module path.
    //
    // Use BTreeMap for each of these so that things stay consistent, regardless of how the
    // metadata is ordered.
    module_path_map: BTreeMap<String, String>,
    // Top level modules
    modules: BTreeMap<String, Module>,
    // Top-level type definitions and functions, keyed by module name
    module_docstrings: BTreeMap<String, String>,
    module_toml: BTreeMap<String, String>,
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
    trait_impls: BTreeMap<(String, String), BTreeMap<String, ObjectTraitImpl>>,
}

impl UniffiMetaConverter {
    /// Add a [uniffi_meta::Metadata] item to be converted
    pub fn add_metadata_item(&mut self, meta: uniffi_meta::Metadata) -> Result<()> {
        match meta {
            uniffi_meta::Metadata::Namespace(namespace) => {
                // Map both the crate name and module name to the module name.  This simplifies the
                // code in `get_module`
                self.module_path_map
                    .insert(namespace.crate_name.clone(), namespace.name.clone());
                self.module_path_map
                    .insert(namespace.name.clone(), namespace.name.clone());
                // Insert a new module
                self.modules
                    .entry(namespace.name.clone())
                    .or_insert(Module {
                        // TODO: Is this the correct crate name in all cases?  I'm pretty sure
                        // it is for proc-macro based generation, since the proc-macro
                        // namespace is added to the metadata list before the UDL one.
                        // However, I'm not so sure what happens if we're generating from a UDL
                        // file.
                        crate_name: namespace.crate_name,
                        docstring: None,
                        config_toml: None,
                        name: namespace.name,
                        functions: vec![],
                        type_definitions: vec![],
                    });
            }
            uniffi_meta::Metadata::Func(func) => {
                self.functions
                    .entry(func.module_path.clone())
                    .or_default()
                    .insert(func.name.clone(), Function::try_from_node(func)?);
            }
            uniffi_meta::Metadata::Record(rec) => {
                self.records
                    .entry(rec.module_path.clone())
                    .or_default()
                    .insert(rec.name.clone(), Record::try_from_node(rec)?);
            }
            uniffi_meta::Metadata::Enum(en) => {
                self.enums
                    .entry(en.module_path.clone())
                    .or_default()
                    .insert(en.name.clone(), Enum::try_from_node(en)?);
            }
            uniffi_meta::Metadata::Object(int) => {
                self.interfaces
                    .entry(int.module_path.clone())
                    .or_default()
                    .insert(int.name.clone(), Interface::try_from_node(int)?);
            }
            uniffi_meta::Metadata::CallbackInterface(cbi) => {
                self.callback_interfaces
                    .entry(cbi.module_path.clone())
                    .or_default()
                    .insert(cbi.name.clone(), CallbackInterface::try_from_node(cbi)?);
            }
            uniffi_meta::Metadata::CustomType(custom) => {
                self.custom_types
                    .entry(custom.module_path.clone())
                    .or_default()
                    .insert(custom.name.clone(), CustomType::try_from_node(custom)?);
            }
            uniffi_meta::Metadata::Constructor(cons) => {
                self.constructors
                    .entry((cons.module_path.clone(), cons.self_name.clone()))
                    .or_default()
                    .insert(cons.name.clone(), Constructor::try_from_node(cons)?);
            }
            uniffi_meta::Metadata::Method(meth) => {
                self.methods
                    .entry((meth.module_path.clone(), meth.self_name.clone()))
                    .or_default()
                    .insert(meth.name.clone(), Method::try_from_node(meth)?);
            }
            uniffi_meta::Metadata::TraitMethod(meth) => {
                self.trait_methods
                    .entry((meth.module_path.clone(), meth.trait_name.clone()))
                    .or_default()
                    .insert(meth.name.clone(), TraitMethod::try_from_node(meth)?);
            }
            uniffi_meta::Metadata::UniffiTrait(ut) => {
                let meth = match &ut {
                    uniffi_meta::UniffiTraitMetadata::Debug { fmt } => fmt,
                    uniffi_meta::UniffiTraitMetadata::Display { fmt } => fmt,
                    uniffi_meta::UniffiTraitMetadata::Eq { eq, .. } => eq,
                    uniffi_meta::UniffiTraitMetadata::Hash { hash } => hash,
                };

                self.uniffi_traits
                    .entry((meth.module_path.clone(), meth.self_name.clone()))
                    .or_default()
                    .insert(ut.name().clone(), UniffiTrait::try_from_node(ut)?);
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
                    .entry((module_path.to_string(), name.to_string()))
                    .or_default()
                    .insert(
                        imp.trait_name.clone(),
                        ObjectTraitImpl {
                            tr_module_name: match &imp.tr_module_path {
                                None => None,
                                Some(module_path) => Some(
                                    get_module_name(&self.module_path_map, module_path)?
                                        .to_string(),
                                ),
                            },
                            ..ObjectTraitImpl::try_from_node(imp)?
                        },
                    );
            }
            uniffi_meta::Metadata::UdlFile(_) => (),
        }
        Ok(())
    }

    pub fn add_module_config_toml(&mut self, module_name: String, path: &Utf8Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let contents =
            fs::read_to_string(path).with_context(|| format!("read file: {:?}", path))?;
        self.module_toml.insert(module_name, contents);
        Ok(())
    }

    /// Add a docstring for a module,
    ///
    /// This is currently UDL-specific.  Eventually, we should probably make this another metadata
    /// items
    pub fn add_module_docstring(&mut self, module_name: String, docstring: String) {
        self.module_docstrings.insert(module_name, docstring);
    }

    pub fn try_into_initial_ir(mut self) -> Result<Root> {
        let mut root = Root {
            modules: self.modules.into_iter().collect(),
            cdylib: None,
        };

        // Move child items into their parents
        for (module_path, docstring) in self.module_docstrings {
            get_module(&self.module_path_map, &mut root, &module_path)?.docstring = Some(docstring);
        }
        for (module_path, toml) in self.module_toml {
            get_module(&self.module_path_map, &mut root, &module_path)?.config_toml = Some(toml);
        }
        for (module_path, funcs) in self.functions {
            get_module(&self.module_path_map, &mut root, &module_path)?
                .functions
                .extend(funcs.into_values());
        }
        for (module_path, list) in self.records {
            get_module(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(list.into_values().map(TypeDefinition::Record));
        }
        for (module_path, list) in self.enums {
            get_module(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(list.into_values().map(TypeDefinition::Enum));
        }
        for (module_path, list) in self.custom_types {
            get_module(&self.module_path_map, &mut root, &module_path)?
                .type_definitions
                .extend(list.into_values().map(TypeDefinition::Custom));
        }
        // Collect child items for interfaces and callback interfaces
        for (module_path, list) in self.interfaces {
            get_module(&self.module_path_map, &mut root, &module_path)?
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
            get_module(&self.module_path_map, &mut root, &module_path)?
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
        // normalize type module names
        root.try_visit_mut(|ty: &mut Type| match ty {
            Type::Interface { module_name, .. }
            | Type::Record { module_name, .. }
            | Type::Enum { module_name, .. }
            | Type::CallbackInterface { module_name, .. }
            | Type::Custom { module_name, .. } => {
                *module_name = get_module_name(&self.module_path_map, module_name)?.to_string();
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

fn get_module_name<'a>(
    module_path_map: &'a BTreeMap<String, String>,
    module_path: &str,
) -> Result<&'a str> {
    module_path_map
        .get(module_path)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("module lookup failed: {module_path:?}"))
}

fn get_module<'a>(
    module_path_map: &BTreeMap<String, String>,
    root: &'a mut Root,
    module_path: &str,
) -> Result<&'a mut Module> {
    let module_name = get_module_name(module_path_map, module_path)?;
    root.modules
        .get_mut(module_name)
        .ok_or_else(|| anyhow!("root module lookup failed: {module_path:?}"))
}
