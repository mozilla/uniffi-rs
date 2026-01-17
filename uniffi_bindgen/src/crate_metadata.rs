/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helpers which allow metadata for crate locations to be specified in TOML.
//! Returns what `cargo_metadata` does, but useful in environments where `cargo_metadata`
//! isn't available or doesn't work.
//!
//! For example, you might have TOML with:
//! ```toml
//! [my_crate]
//! crate_root = "path/to/my_crate"
//!
//! [other_crate]
//! config = "path/to/other_crate/other.toml"
//! "other_crate.udl" = "path/to/other_crate/src/other_crate.udl"
//! ```
//! In the first example, `uniffi.toml` and any requested UDL files will be located relative to the `crate_root`
//! In the second example, the explicit paths to files are specified.
//! In all cases, paths can be relative to the TOML file itself, or absolute.

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use std::fs;

use crate::BindgenPathsLayer;

#[derive(Debug, Clone, Default)]
pub struct TomlCrateConfigSupplier {
    // any relative paths in the TOML will be considered relative to this.
    root: Utf8PathBuf,
    // the content of the .toml file we loaded.
    toml: toml::Table,
}

impl TomlCrateConfigSupplier {
    pub fn new(path: &Utf8PathBuf) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read file: {:?}", path))?;
        let toml =
            toml::de::from_str(&contents).with_context(|| format!("parse toml: {:?}", path))?;
        let root = path
            .parent()
            .ok_or_else(|| anyhow!(format!("No parent parent of {path:?}")))?
            .to_owned();
        Ok(Self { root, toml })
    }

    fn make_path(&self, path_val: &toml::Value) -> Result<Utf8PathBuf> {
        let path_str = path_val
            .as_str()
            .ok_or(anyhow!("toml value is not a string"))?;
        Ok(Utf8PathBuf::from(&self.root).join(path_str))
    }
}

impl BindgenPathsLayer for TomlCrateConfigSupplier {
    fn get_config(&self, crate_name: &str) -> Result<Option<toml::value::Table>> {
        // The toml can just specify the crate root, or individually each of the config and udl.
        let Some(crate_metadata) = self.toml.get(crate_name) else {
            return Ok(None);
        };

        let config_path = match crate_metadata.get("config") {
            Some(config_path) => self
                .make_path(config_path)
                .with_context(|| format!("loading `config` entry for crate `{crate_name}`"))?,
            None => {
                let Some(crate_root) = crate_metadata.get("crate_root") else {
                    return Ok(None);
                };
                let config_path = self
                    .make_path(crate_root)
                    .with_context(|| {
                        format!("loading `crate_root` entry for crate `{crate_name}`")
                    })?
                    .join("uniffi.toml");
                if !config_path.exists() {
                    return Ok(None);
                }
                config_path
            }
        };
        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("read file: {:?}", config_path))?;
        let toml = toml::de::from_str(&contents)
            .with_context(|| format!("parse toml: {:?}", config_path))?;
        Ok(Some(toml))
    }

    fn get_udl_path(&self, crate_name: &str, udl_name: &str) -> Option<Utf8PathBuf> {
        // The toml can just specify the crate root, or individually each of the config and udl.
        let crate_metadata = self.toml.get(crate_name)?;
        match crate_metadata.get(udl_name) {
            Some(path) => self
                .make_path(path)
                .with_context(|| format!("loading `{udl_name}` entry for crate `{crate_name}`"))
                .ok(),
            None => match crate_metadata.get("crate_root") {
                Some(p) => Some(
                    self.make_path(p)
                        .with_context(|| {
                            format!("locating udl via `crate_root` entry for `{crate_name}`")
                        })
                        .ok()?
                        .join("src")
                        .join(format!("{udl_name}.udl")),
                ),
                None => None,
            },
        }
    }
}
