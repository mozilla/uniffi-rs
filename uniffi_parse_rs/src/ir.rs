/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use camino::Utf8Path;
use quote::format_ident;
use syn::{ext::IdentExt, Ident};
use uniffi_meta::MetadataGroup;

use crate::{Item, MetadataGroupMap, Module, Result};

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
    pub fn add_crate_root(&mut self, crate_name: &str, file_path: &Utf8Path) -> Result<&Module> {
        let crate_name = crate_name.replace("-", "_");
        let ident = format_ident!("{crate_name}");
        let module = Module::new_crate_root(ident.clone(), file_path)?;
        self.crate_roots.insert(ident.clone(), Item::Module(module));
        match self.crate_roots.get(&ident) {
            Some(Item::Module(m)) => Ok(m),
            _ => unreachable!(),
        }
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

    pub fn crate_roots_mut(&mut self) -> impl Iterator<Item = &mut Module> {
        self.crate_roots.values_mut().map(|item| match item {
            Item::Module(module) => module,
            item => panic!("Crate root is not Item::Module ({item:?})"),
        })
    }

    pub fn into_metadata_group_map(self) -> Result<MetadataGroupMap> {
        self.crate_roots()
            .map(|module| {
                let mut group = MetadataGroup {
                    namespace: uniffi_meta::NamespaceMetadata {
                        crate_name: module.ident.unraw().to_string(),
                        name: module.ident.unraw().to_string(),
                    },
                    namespace_docstring: module.docstring.clone(),
                    items: BTreeSet::default(),
                };
                module.create_metadata(&mut group)?;
                Ok((module.ident.unraw().to_string(), group))
            })
            .collect()
    }

    #[cfg(test)]
    pub fn new_for_test(test_sources: &[&str]) -> Self {
        Self::new_for_test_with_env(test_sources)
    }

    #[cfg(test)]
    pub fn new_for_test_with_env(test_sources: &[&str]) -> Self {
        let mut ir = Self::default();
        for test_source in test_sources.iter() {
            ir.add_crate_root(
                test_source,
                &camino::Utf8PathBuf::from(format!("src/test_src/{test_source}.rs")),
            )
            .unwrap();
        }
        ir
    }
}
