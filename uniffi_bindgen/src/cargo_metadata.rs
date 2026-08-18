/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers for data returned by cargo_metadata. Note that this doesn't
//! execute cargo_metadata, just parses its output.

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::Package;
use std::collections::{HashMap, HashSet};

use crate::{bindgen_paths::SourceCrate, BindgenPathsLayer, CargoMetadataOptions};

#[derive(Debug, Clone, Default)]
pub struct CrateConfigSupplier {
    /// Map crates names to Package instances
    package_map: HashMap<String, Package>,
    /// Map library target names to Package instances
    lib_to_package_map: HashMap<String, Package>,
    options: CargoMetadataOptions,
}

impl CrateConfigSupplier {
    pub fn from_cargo_metadata_command(
        cmd: cargo_metadata::MetadataCommand,
        options: CargoMetadataOptions,
    ) -> Result<Self> {
        Ok(Self::from_cargo_metadata(
            cmd.exec().context("error running cargo metadata")?,
            options,
        ))
    }

    pub fn from_cargo_metadata(
        metadata: cargo_metadata::Metadata,
        options: CargoMetadataOptions,
    ) -> Self {
        let lib_to_package_map = metadata
            .packages
            .iter()
            .cloned()
            .filter_map(|p| {
                let target = p.targets.iter().find(|t| {
                    !t.is_bin()
                        && !t.is_example()
                        && !t.is_test()
                        && !t.is_bench()
                        && !t.is_custom_build()
                })?;
                Some((target.name.replace('-', "_"), p))
            })
            .collect();
        Self {
            package_map: metadata
                .packages
                .into_iter()
                .map(|p| (p.name.replace("-", "_"), p))
                .collect(),
            lib_to_package_map,
            options,
        }
    }

    fn get_package(&self, crate_name: &str) -> Option<&Package> {
        self.package_map.get(&crate_name.replace("-", "_"))
    }

    fn get_package_by_library_name(&self, library_name: &str) -> Option<&Package> {
        self.lib_to_package_map.get(&library_name.replace("-", "_"))
    }

    fn get_crate_root(&self, crate_name: &str) -> Option<&Utf8Path> {
        // Some code uses the library name and other code uses the package name to identify crates,
        // let's try with both.
        let pkg = self
            .get_package(crate_name)
            .or_else(|| self.get_package_by_library_name(crate_name))?;
        Some(pkg.manifest_path.parent().unwrap())
    }
}

// Newer trait for finding crate roots and paths
impl BindgenPathsLayer for CrateConfigSupplier {
    fn get_crate_root(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.get_crate_root(crate_name).map(Utf8Path::to_path_buf)
    }

    fn get_cdylib_name(&self, crate_name: &str) -> Option<String> {
        let pkg = self.get_package(crate_name)?;
        pkg.targets
            .iter()
            .find(|t| t.is_cdylib())
            .map(|t| t.name.clone())
    }

    fn get_source_crates(&self, source_crate: &str) -> Option<Vec<SourceCrate>> {
        // Packages to process
        let mut todo_stack = vec![source_crate];
        // Packages we've already processed
        let mut seen = HashSet::new();
        // Map crate names to features explicitly enabled
        //
        // We need to calculate features ourselves because of
        // https://github.com/rust-lang/cargo/issues/7754
        let mut feature_map: HashMap<&str, HashSet<&str>> = HashMap::new();
        // Map crate names to default features
        let mut default_feature_map: HashMap<&str, HashSet<&str>> = HashMap::new();
        // Should we enable default features for a crate?
        let mut enable_default_features: HashSet<&str> = HashSet::new();
        // Source crates found
        let mut found = vec![];

        while let Some(crate_name) = todo_stack.pop() {
            if !seen.insert(crate_name) {
                continue;
            }
            let Some(package) = self.get_package(crate_name) else {
                continue;
            };
            if should_parse_package(package) {
                if let Some(src_path) = find_src_lib(package) {
                    found.push((package.name.to_string(), src_path.clone()));
                }
            }

            for dep in package.dependencies.iter() {
                feature_map
                    .entry(dep.name.as_str())
                    .or_default()
                    .extend(dep.features.iter().map(|s| s.as_str()));
            }

            if let Some(pkg_default_features) = package.features.get("default") {
                default_feature_map.insert(
                    crate_name,
                    pkg_default_features.iter().map(|s| s.as_str()).collect(),
                );
            }
            if crate_name == source_crate {
                let package_features = package
                    .features
                    .keys()
                    .map(|f| f.as_str())
                    .filter(|f| *f != "default");
                if self.options.all_features {
                    feature_map
                        .entry(crate_name)
                        .or_default()
                        .extend(package_features);
                } else {
                    feature_map
                        .entry(crate_name)
                        .or_default()
                        .extend(package_features.filter(|f| {
                            self.options.features.iter().any(|selected| selected == f)
                        }));
                }
                if !self.options.no_default_features {
                    enable_default_features.insert(crate_name);
                }
            }

            for dep in package.dependencies.iter() {
                if dep.uses_default_features {
                    enable_default_features.insert(dep.name.as_str());
                }
                todo_stack.push(dep.name.as_str());
            }
        }

        Some(
            found
                .into_iter()
                .map(|(name, src_path)| {
                    let mut features = vec![];
                    if let Some(f) = feature_map.get(name.as_str()) {
                        features.extend(f.iter().map(|s| s.to_string()));
                    }
                    if enable_default_features.contains(name.as_str()) {
                        if let Some(f) = default_feature_map.get(name.as_str()) {
                            features.extend(f.iter().map(|s| s.to_string()));
                        }
                    }
                    SourceCrate {
                        features,
                        name,
                        src_path,
                    }
                })
                .collect(),
        )
    }
}

fn should_parse_package(package: &Package) -> bool {
    let skip_parsing = package
        .metadata
        .pointer("/uniffi/skip_parsing")
        .and_then(|v| v.as_bool())
        .unwrap_or_default();
    let depends_on_uniffi = package.dependencies.iter().any(|d| d.name == "uniffi");

    depends_on_uniffi && !skip_parsing
}

fn find_src_lib(package: &Package) -> Option<Utf8PathBuf> {
    package
        .targets
        .iter()
        .find(|t| t.is_lib() || t.is_rlib() || t.is_cdylib() || t.is_staticlib())
        .map(|t| t.src_path.clone())
}
