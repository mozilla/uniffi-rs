/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers for data returned by cargo_metadata. Note that this doesn't
//! execute cargo_metadata, just parses its output.

use anyhow::{bail, Context};
use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use std::{collections::HashMap, fs};

use crate::BindgenCrateConfigSupplier;

#[derive(Debug, Clone, Default)]
pub struct CrateConfigSupplier {
    paths: HashMap<String, Utf8PathBuf>,
}

impl BindgenCrateConfigSupplier for CrateConfigSupplier {
    fn get_toml(&self, crate_name: &str) -> anyhow::Result<Option<toml::value::Table>> {
        let toml = self.paths.get(crate_name).map(|p| p.join("uniffi.toml"));
        crate::load_toml_file(toml.as_deref())
    }

    fn get_udl(&self, crate_name: &str, udl_name: &str) -> anyhow::Result<String> {
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
                p.targets.iter().filter(|t| t.is_lib()).filter_map(|t| {
                    p.manifest_path
                        .parent()
                        .map(|p| (t.name.replace('-', "_"), p.to_owned()))
                })
            })
            .collect();
        Self { paths }
    }
}
