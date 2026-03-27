/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use camino::Utf8Path;
use quote::format_ident;
use syn::{ext::IdentExt, Ident};
use uniffi_meta::MetadataGroup;

use crate::{
    paths::LookupCache, CompileEnv, Error, ErrorKind::*, Item, MetadataGroupMap, Module, RPath,
    Result,
};

/// Intermediate representation of the interface
///
/// The first parsing step converts `syn` types and stores them here.
/// The main goal is to parse enough of `syn` to resolve `syn::Type` into `uniffi_meta::Type`.
#[derive(Default, Debug)]
pub struct Ir {
    /// Map idents to the crate root modules
    ///
    /// Every value is `Item::Module`
    pub crate_roots: HashMap<Ident, Item>,
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
        env: &CompileEnv,
    ) -> Result<&Module> {
        let crate_name = crate_name.replace("-", "_");
        let ident = format_ident!("{crate_name}");
        let module = Module::new_crate_root(ident.clone(), file_path, env)?;
        self.crate_roots.insert(ident.clone(), Item::Module(module));
        match self.crate_roots.get(&ident) {
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
        self.crate_roots.get(ident).map(|item| match item {
            Item::Module(item) => item,
            item => panic!("Crate root is not Item::Module ({item:?})"),
        })
    }

    pub fn crate_roots(&self) -> impl Iterator<Item = &Module> {
        self.crate_roots.values().map(|item| match item {
            Item::Module(module) => module,
            item => panic!("Crate root is not Item::Module ({item:?})"),
        })
    }

    pub fn crate_roots_and_paths(&self) -> impl Iterator<Item = (RPath<'_>, &Module)> {
        self.crate_roots.values().map(|item| match item {
            Item::Module(module) => (RPath::new(item), module),
            item => panic!("Crate root is not Item::Module ({item:?})"),
        })
    }

    pub fn crate_roots_mut(&mut self) -> impl Iterator<Item = &mut Module> {
        self.crate_roots.values_mut().map(|item| match item {
            Item::Module(module) => module,
            item => panic!("Crate root is not Item::Module ({item:?})"),
        })
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
                &env,
            )
            .unwrap();
        }
        crate::resolve_macros(&mut ir).unwrap();
        ir
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
