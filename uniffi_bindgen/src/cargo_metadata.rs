/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers for data returned by cargo_metadata. Note that this doesn't
//! execute cargo_metadata, just parses its output.

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use std::{collections::HashMap, fs};

use crate::BindgenCrateConfigSupplier;

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

impl BindgenCrateConfigSupplier for CrateConfigSupplier {
    fn get_toml(&self, crate_name: &str) -> Result<Option<toml::value::Table>> {
        crate::load_toml_file(self.get_toml_path(crate_name).as_deref())
    }

    fn get_toml_path(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.paths.get(crate_name).map(|p| p.join("uniffi.toml"))
    }

    fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        let path = self
            .paths
            .get(crate_name)
            .context(format!("No path known to UDL files for '{crate_name}'"))?
            .join("src")
            .join(format!("{udl_name}.udl"));
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
