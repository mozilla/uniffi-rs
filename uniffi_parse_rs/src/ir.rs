/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use camino::Utf8Path;
use quote::format_ident;
use syn::{ext::IdentExt, Ident};
use uniffi_meta::MetadataGroup;

use crate::{
    attrs::{
        EnumAttributes, FunctionAttributes, ImplAttributes, ObjectAttributes, RecordAttributes,
        TraitAttributes,
    },
    macros::maybe_resolve_macro,
    paths::LookupCache,
    CompileEnv, Enum, Error,
    ErrorKind::*,
    Function, Impl, Item, MetadataGroupMap, Module, Object, RPath, Record, Result, Trait,
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
    compile_env: CompileEnv,
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
        let module = match self.crate_roots_mut().find(|c| c.ident == crate_name) {
            Some(m) => m,
            None => return Err(Error::new_without_location(NotFound)),
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

    /// Resolve Item::Unresolved to more specific items like Item::UseRemoteType
    ///
    /// This needs to run after adding all crates parsing, since it requires looking up module paths.
    pub fn resolve_items(&mut self) -> syn::Result<()> {
        // Find and remove unresolved items from all modules
        let mut unresolved_map = HashMap::new();
        for crate_root in self.crate_roots.values_mut() {
            crate_root.visit_modules_mut(|module| {
                unresolved_map.insert(module.id, module.remove_unresolved_items());
            });
        }

        // Resolve the items
        let mut resolved_map = HashMap::new();
        let mut cache = LookupCache::default();
        for crate_root in self.crate_roots.values() {
            crate_root.try_visit_modules_and_paths(|module, rpath| {
                let Some(unresolved) = unresolved_map.remove(&module.id) else {
                    return Ok(());
                };

                let mut resolved_items = vec![];
                for item in unresolved {
                    if let Some(item) =
                        self.resolve_item(&mut cache, rpath, &crate_root.compile_env, item)?
                    {
                        resolved_items.push(item);
                    }
                }
                resolved_map.insert(module.id, resolved_items);
                Ok(())
            })?;
        }

        // Add resolved items to the modules
        for crate_root in self.crate_roots.values_mut() {
            crate_root.visit_modules_mut(|module| {
                if let Some(resolved) = resolved_map.remove(&module.id) {
                    module.items.extend(resolved);
                }
            });
        }
        Ok(())
    }

    /// Resolve an unresolved item
    fn resolve_item<'ir>(
        &'ir self,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        env: &CompileEnv,
        item: syn::Item,
    ) -> syn::Result<Option<Item>> {
        Ok(match item {
            syn::Item::Fn(func) => {
                if let Some(attrs) =
                    FunctionAttributes::parse(self, cache, module_path, env, &func.attrs)?
                {
                    Some(Item::Fn(Function::parse(attrs, func.clone())?))
                } else {
                    None
                }
            }
            syn::Item::Struct(st) => {
                if let Some(attrs) =
                    RecordAttributes::parse(self, cache, module_path, env, &st.attrs)?
                {
                    let r = Record::parse(env, attrs, st)?;
                    Some(Item::Record(r))
                } else if let Some(attrs) =
                    ObjectAttributes::parse(self, cache, module_path, env, &st.attrs)?
                {
                    Some(Item::Object(Object::parse(attrs, st.ident, st.vis)?))
                } else {
                    Some(Item::NonUniffi(st.vis.into(), st.ident))
                }
            }
            syn::Item::Enum(en) => {
                if let Some(attrs) =
                    ObjectAttributes::parse(self, cache, module_path, env, &en.attrs)?
                {
                    Some(Item::Object(Object::parse(attrs, en.ident, en.vis)?))
                } else if let Some(attrs) =
                    EnumAttributes::parse(self, cache, module_path, env, &en.attrs)?
                {
                    Some(Item::Enum(Enum::parse(env, attrs, en)?))
                } else {
                    Some(Item::NonUniffi(en.vis.into(), en.ident))
                }
            }
            syn::Item::Trait(tr) => {
                if let Some(attrs) =
                    TraitAttributes::parse(self, cache, module_path, env, &tr.attrs)?
                {
                    Some(Item::Trait(Trait::parse(env, attrs, tr)?))
                } else {
                    None
                }
            }
            syn::Item::Impl(imp) => {
                if let Some(attrs) =
                    ImplAttributes::parse(self, cache, module_path, env, &imp.attrs)?
                {
                    Some(Item::Impl(Impl::parse(env, attrs, imp)?))
                } else {
                    None
                }
            }
            syn::Item::Macro(mac) => maybe_resolve_macro(self, cache, module_path, &mac)?,
            _ => None,
        })
    }

    #[cfg(test)]
    pub fn new_for_test(test_sources: &[&str]) -> Self {
        let mut ir = Self::new_for_test_with_env(test_sources, CompileEnv::new_for_test());
        ir.resolve_items().expect("resolve_items failed");
        ir
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

    // Couple of helper modules for `Ir::resolve_items`
    //
    // There's more combinations that could be implemented, but no need yet.

    fn try_visit_modules_and_paths<'ir>(
        &'ir self,
        mut visitor: impl FnMut(&'ir Module, &RPath<'ir>) -> syn::Result<()>,
    ) -> syn::Result<()> {
        let mut stack = vec![(self.module(), RPath::new(&self.module))];
        while let Some((module, path)) = stack.pop() {
            visitor(module, &path)?;
            for item in module.items.iter() {
                if let Item::Module(m) = item {
                    stack.push((m, path.append_child(item)));
                }
            }
        }
        Ok(())
    }

    fn visit_modules_mut(&mut self, mut visitor: impl FnMut(&mut Module)) {
        let mut stack = vec![self.module_mut()];
        while let Some(module) = stack.pop() {
            visitor(module);
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
