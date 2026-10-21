/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use camino::Utf8Path;
use quote::format_ident;
use syn::{ext::IdentExt, Ident};
use uniffi_meta::MetadataGroup;

use crate::{
    items::resolve_simple_items, macros::resolve_uniffi_macros, paths::LookupCache,
    trait_impls::resolve_trait_items, CompileEnv, Error, ErrorKind::*, Item, MetadataGroupMap,
    Module, RPath, Result,
};

/// Intermediate representation of the interface
///
/// The first parsing step converts `syn` types and stores them here.
/// The main goal is to parse enough of `syn` to resolve `syn::Type` into `uniffi_meta::Type`.
#[derive(Default)]
pub struct Ir {
    pub crate_roots: HashMap<Ident, CrateRoot>,
}

pub struct CrateRoot {
    /// Root module, represented as an `Item::Module`.
    ///
    /// This is an `Item::Module` because of `RPath`.
    /// That type wants to store a `Vec<&Item>` for each component of the path.
    module: Item,
    pub compile_env: CompileEnv,
}

impl Ir {
    pub fn new() -> Self {
        Self {
            crate_roots: HashMap::new(),
        }
    }

    /// Add a new crate to the IR
    pub fn add_crate_root(
        &mut self,
        crate_name: &str,
        file_path: &Utf8Path,
        compile_env: CompileEnv,
    ) -> Result<&Module> {
        let crate_name = crate_name.replace("-", "_");
        let ident = format_ident!("{crate_name}");
        let module = Module::new_crate_root(ident.clone(), file_path, &compile_env)?;
        self.crate_roots.insert(
            ident.clone(),
            CrateRoot {
                module: Item::Module(module),
                compile_env,
            },
        );
        match self
            .crate_roots
            .get(&ident)
            .map(|crate_root| &crate_root.module)
        {
            Some(Item::Module(m)) => Ok(m),
            _ => unreachable!(),
        }
    }

    pub fn add_udl_metadata(
        &mut self,
        crate_name: &str,
        items: impl IntoIterator<Item = uniffi_meta::Metadata>,
    ) -> Result<()> {
        let module = match self
            .crate_roots_mut()
            .find(|c| c.ident == crate_name.replace("-", "_"))
        {
            Some(m) => m,
            None => {
                return Err(Error::new_without_location(AddUdlMetadataError(format!(
                    "Invalid crate: {crate_name}"
                ))))
            }
        };
        for i in items {
            match &i {
                uniffi_meta::Metadata::Enum(e) => {
                    module.items.push(Item::Udl(uniffi_meta::Type::Enum {
                        module_path: e.module_path.clone(),
                        name: e.name.clone(),
                    }));
                }
                uniffi_meta::Metadata::Record(r) => {
                    module.items.push(Item::Udl(uniffi_meta::Type::Record {
                        module_path: r.module_path.clone(),
                        name: r.name.clone(),
                    }));
                }
                uniffi_meta::Metadata::Object(o) => {
                    module.items.push(Item::Udl(uniffi_meta::Type::Object {
                        module_path: o.module_path.clone(),
                        name: o.name.clone(),
                        imp: o.imp,
                    }));
                }
                uniffi_meta::Metadata::CallbackInterface(c) => {
                    module
                        .items
                        .push(Item::Udl(uniffi_meta::Type::CallbackInterface {
                            module_path: c.module_path.clone(),
                            name: c.name.clone(),
                        }));
                }
                uniffi_meta::Metadata::CustomType(c) => {
                    module.items.push(Item::Udl(uniffi_meta::Type::Custom {
                        module_path: c.module_path.clone(),
                        name: c.name.clone(),
                        builtin: Box::new(c.builtin.clone()),
                    }));
                }
                _ => (),
            }
            module.metadata_from_udl.push(i);
        }
        Ok(())
    }

    pub fn crate_root(&self, ident: &Ident) -> Option<&Module> {
        self.crate_roots.get(ident).map(CrateRoot::module)
    }

    pub fn crate_roots(&self) -> impl Iterator<Item = &Module> {
        self.crate_roots.values().map(CrateRoot::module)
    }

    pub fn crate_roots_and_paths(&self) -> impl Iterator<Item = (RPath<'_>, &Module)> {
        self.crate_roots
            .values()
            .map(|crate_root| (RPath::new(&crate_root.module), crate_root.module()))
    }

    pub fn crate_roots_mut(&mut self) -> impl Iterator<Item = &mut Module> {
        self.crate_roots.values_mut().map(CrateRoot::module_mut)
    }

    pub fn into_metadata_group_map(self) -> Result<MetadataGroupMap> {
        let mut cache = LookupCache::default();
        self.crate_roots_and_paths()
            .map(|(mut module_path, module)| {
                let mut group = MetadataGroup {
                    namespace: uniffi_meta::NamespaceMetadata {
                        crate_name: module.ident.unraw().to_string(),
                        name: module.ident.unraw().to_string(),
                    },
                    namespace_docstring: module.docstring.clone(),
                    items: BTreeSet::default(),
                };
                module.create_metadata(&self, &mut cache, &mut module_path, &mut group)?;
                Ok((module.ident.unraw().to_string(), group))
            })
            .collect()
    }

    /// Resolve syn items into `Item` instances and add them to the IR
    ///
    /// This needs to run after adding all crates parsing, since it requires looking up module paths.
    pub fn resolve_items(&mut self) -> Result<()> {
        // Simple items come first
        resolve_simple_items(self)?;
        // UniFFI macros next.
        //
        // We need to lookup the Rust type for `custom_type!` calls, which is handled by `resolve_simple_items`
        resolve_uniffi_macros(self)?;
        // Last is trait impls
        //
        // We need to look up which type the trait was implemented for,
        // which is handled by the previous items
        resolve_trait_items(self)?;
        Ok(())
    }

    /// Visit modules and their unresolved items
    ///
    /// Calls the visitor with (crate_root, module, module_path, unresolved_item) tuples.
    ///
    /// If visitors handle the unresolved_item, they should return `VisitUnresolvedItem::Remove` to
    /// remove it from the list.
    pub fn visit_unresolved_items<'ir>(
        &'ir self,
        mut visitor: impl FnMut(&CrateRoot, &'ir Module, &RPath<'ir>, &syn::Item) -> VisitUnresolvedItem,
    ) {
        self.try_visit_unresolved_items(|crate_root, module, path, unresolved| {
            Ok(visitor(crate_root, module, path, unresolved))
        })
        .unwrap()
    }

    /// Failable version of `visit_unresolved_items`
    pub fn try_visit_unresolved_items<'ir>(
        &'ir self,
        mut visitor: impl FnMut(
            &CrateRoot,
            &'ir Module,
            &RPath<'ir>,
            &syn::Item,
        ) -> Result<VisitUnresolvedItem>,
    ) -> Result<()> {
        for crate_root in self.crate_roots.values() {
            let mut stack = vec![(crate_root.module(), RPath::new(&crate_root.module))];
            while let Some((module, path)) = stack.pop() {
                // take the unresolved items so we can process them
                let mut unresolved = module.unresolved_items.take();
                let mut err = None;
                unresolved.retain(|unresolved_item| {
                    if err.is_some() {
                        return true;
                    }
                    match visitor(crate_root, module, &path, unresolved_item) {
                        Ok(VisitUnresolvedItem::Keep) => true,
                        Ok(VisitUnresolvedItem::Remove) => false,
                        Err(e) => {
                            // visit failed, keep the rest of the items
                            err = Some(e);
                            true
                        }
                    }
                });
                module.extend_unresolved_items(unresolved);

                // Return errors now
                if let Some(e) = err {
                    return Err(e);
                }

                // find child modules to visit
                for item in module.items.iter() {
                    if let Item::Module(m) = item {
                        stack.push((m, path.append_child(item)));
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn new_for_test(test_sources: &[&str]) -> Self {
        Self::new_for_test_with_env(test_sources, CompileEnv::new_for_test())
    }

    #[cfg(test)]
    pub fn new_for_test_with_env(test_sources: &[&str], env: CompileEnv) -> Self {
        let mut ir = Self::default();
        for test_source in test_sources.iter() {
            ir.add_crate_root(
                test_source,
                &camino::Utf8PathBuf::from(format!("src/test_src/{test_source}.rs")),
                env.clone(),
            )
            .unwrap();
        }
        ir.resolve_items().expect("resolve_items failed");
        ir
    }
}

pub enum VisitUnresolvedItem {
    /// Keep the unresolved item for a later pass
    Keep,
    /// Remove the unresolved item after resolving it
    Remove,
}

impl CrateRoot {
    pub fn module_item(&self) -> &Item {
        &self.module
    }

    pub fn module(&self) -> &Module {
        match &self.module {
            Item::Module(module) => module,
            item => panic!("crate_root.module is not Item::Module ({item:?})"),
        }
    }

    pub fn module_mut(&mut self) -> &mut Module {
        match &mut self.module {
            Item::Module(module) => module,
            item => panic!("crate_root.module is not Item::Module ({item:?})"),
        }
    }
}

/// Set of IR changes that we can apply later
///
/// The main use for this is to collect a bunch of changes inside a `try_visit_unresolved_items`
/// closure where we have a shared reference to the IR, then apply those changes later when we can
/// take a mutable reference.
#[derive(Default)]
pub struct IrChanges {
    // Map module ids to items to add
    items: HashMap<usize, Vec<Item>>,
}

impl IrChanges {
    pub fn add(&mut self, module_id: usize, item: Item) {
        self.items.entry(module_id).or_default().push(item);
    }

    pub fn apply(mut self, ir: &mut Ir) {
        let mut stack: Vec<&mut Module> = ir
            .crate_roots
            .values_mut()
            .map(CrateRoot::module_mut)
            .collect();
        while let Some(module) = stack.pop() {
            if let Some(items) = self.items.remove(&module.id) {
                module.items.extend(items);
            }
            for item in module.items.iter_mut() {
                if let Item::Module(m) = item {
                    stack.push(m);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeSet, str::FromStr};

    use target_lexicon::Triple;
    use uniffi_meta::Metadata;

    use crate::{CompileEnv, Ir};

    #[test]
    fn test_create_metadata() {
        let ir = Ir::new_for_test(&["full_interface"]);
        let metadata_group_map = ir.into_metadata_group_map().unwrap();
        let metadata_group = metadata_group_map.get("full_interface").unwrap();
        assert_eq!(
            metadata_group.namespace_docstring,
            Some("Module docstring".into())
        );
        let expected = expect_test::expect_file!["./expect/full_interface.txt"];
        expected.assert_eq(&format!("{:#?}", metadata_group.items));
    }

    fn feature_detection_items(target: &str, features: &[&str]) -> BTreeSet<String> {
        let target = Triple::from_str(target).unwrap();
        let features = features.iter().map(|s| s.to_string()).collect();
        let env = CompileEnv::new(target, features);
        let ir = Ir::new_for_test_with_env(&["feature_detection"], env);
        let mut metadata_group_map = ir.into_metadata_group_map().unwrap();
        let metadata_group = metadata_group_map.remove("feature_detection").unwrap();
        let mut items = BTreeSet::default();
        for item in metadata_group.items.into_iter() {
            match item {
                Metadata::Func(f) => {
                    items.insert(format!("func:{}", f.name));
                }
                Metadata::Object(o) => {
                    items.insert(format!("obj:{}", o.name));
                }
                Metadata::Method(m) => {
                    items.insert(format!("method:{}:{}", m.self_name, m.name));
                }
                Metadata::CallbackInterface(c) => {
                    items.insert(format!("cbi:{}", c.name));
                }
                Metadata::Record(r) => {
                    items.insert(format!("rec:{}", r.name));
                    for f in r.fields {
                        items.insert(format!("field:{}:{}", r.name, f.name));
                    }
                }
                Metadata::Enum(e) => {
                    if e.shape.is_error() {
                        items.insert(format!("error:{}", e.name));
                    } else {
                        items.insert(format!("enum:{}", e.name));
                    }
                    for v in e.variants {
                        items.insert(format!("variant:{}:{}", e.name, v.name));
                    }
                }
                Metadata::CustomType(c) => {
                    items.insert(format!("custom_type:{}", c.name));
                }
                _ => (),
            }
        }
        items
    }

    #[test]
    fn test_feature_detection() {
        assert_eq!(
            feature_detection_items("x86_64-unknown-linux-gnu", &["feature1"]),
            BTreeSet::from([
                "rec:Feature1".into(),
                "func:feature2_or_x86_64".into(),
                "func:feature2_xor_x86_64".into(),
                "func:renamed_feature1".into(),
                "enum:EnumOrError".into(),
                "obj:Object".into(),
            ]),
        );
        assert_eq!(
            feature_detection_items("x86_64-unknown-linux-gnu", &["feature2"]),
            BTreeSet::from([
                "enum:NotFeature1".into(),
                "func:feature2_and_x86_64".into(),
                "func:feature2_or_x86_64".into(),
                "func:renamed_no_feature1".into(),
                "error:EnumOrError".into(),
                "obj:Object".into(),
            ]),
        );
        assert_eq!(
            feature_detection_items("aarch64-apple-darwin", &["feature2", "feature3"]),
            BTreeSet::from([
                "enum:NotFeature1".into(),
                "variant:NotFeature1:Feature3".into(),
                "func:feature2_xor_x86_64".into(),
                "func:feature2_or_x86_64".into(),
                "func:renamed_no_feature1".into(),
                "error:EnumOrError".into(),
                "obj:Object".into(),
            ]),
        );
        assert_eq!(
            feature_detection_items("x86_64-unknown-linux-gnu", &["feature1", "feature2"]),
            BTreeSet::from([
                "rec:Feature1".into(),
                "field:Feature1:feature_2".into(),
                "func:feature2_or_x86_64".into(),
                "func:feature2_and_x86_64".into(),
                "func:renamed_feature1".into(),
                "enum:EnumOrError".into(),
                "obj:Object".into(),
                "method:Object:feature_1_and_2".into(),
            ]),
        );
    }

    #[test]
    fn test_udl_path() {
        let ir = Ir::new_for_test(&["udl_include"]);
        let module = ir.crate_roots().find(|m| m.ident == "udl_include").unwrap();
        assert_eq!(module.udl_name, Some("test_udl_name".into()));
    }
}
