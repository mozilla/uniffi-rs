/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers for data returned by cargo_metadata. Note that this doesn't
//! execute cargo_metadata, just parses its output.

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use std::{collections::HashMap, fs};

use crate::{BindgenCrateConfigSupplier, BindgenPathsLayer};

#[derive(Debug, Clone, Default)]
pub struct CrateConfigSupplier {
    paths: HashMap<String, Utf8PathBuf>,
}

impl CrateConfigSupplier {
    /// Create a new `CrateConfigSupplier` by running `cargo metadata`
    pub fn from_cargo_metadata_command(no_deps: bool) -> Result<Self> {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if no_deps {
            cmd.no_deps();
        }
        let metadata = cmd.exec().context("error running cargo metadata")?;
        Ok(Self::from(metadata))
    }
}

// Newer trait for finding config - it's up to the implementation whether this
// is loaded from a file or something else.
impl BindgenPathsLayer for CrateConfigSupplier {
    fn get_config(&self, crate_name: &str) -> Result<Option<toml::value::Table>> {
        let crate_root = self.paths.get(crate_name);
        let Some(crate_root) = crate_root else {
            return Ok(None);
        };
        // Config files are optional so we return None if the file doesn't exist.
        let config_path = crate_root.join("uniffi.toml");
        if !config_path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("read file: {:?}", config_path))?;
        let toml = toml::de::from_str(&contents)
            .with_context(|| format!("parse toml: {:?}", config_path))?;
        Ok(Some(toml))
    }

    fn get_udl_path(&self, crate_name: &str, udl_name: &str) -> Option<Utf8PathBuf> {
        self.paths
            .get(crate_name)
            .map(|p| p.join("src").join(format!("{udl_name}.udl")))
    }
}

// Older trait for finding config paths
impl BindgenCrateConfigSupplier for CrateConfigSupplier {
    fn get_toml(&self, crate_name: &str) -> Result<Option<toml::value::Table>> {
        self.get_config(crate_name)
    }

    fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        let path = self
            .get_udl_path(crate_name, udl_name)
            .context(format!("No path known to UDL files for '{crate_name}'"))?;
        if path.exists() {
            Ok(fs::read_to_string(path)?)
        } else {
            bail!(format!("No UDL file found at '{path}'"));
        }
    }
}

impl From<Metadata> for CrateConfigSupplier {
    fn from(metadata: Metadata) -> Self {
        let paths: HashMap<String, Utf8PathBuf> = metadata
            .packages
            .iter()
            .flat_map(|p| {
                p.targets
                    .iter()
                    .filter(|t| {
                        !t.is_bin()
                            && !t.is_example()
                            && !t.is_test()
                            && !t.is_bench()
                            && !t.is_custom_build()
                    })
                    .filter_map(|t| {
                        p.manifest_path
                            .parent()
                            .map(|p| (t.name.replace('-', "_"), p.to_owned()))
                    })
            })
            .collect();
        Self { paths }
    }
}
