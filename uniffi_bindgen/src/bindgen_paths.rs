/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;

use anyhow::{bail, Context};
use camino::Utf8PathBuf;

use crate::Result;

/// Responsible for looking up UDL and config paths
///
/// This uses a layered approach supporting multiple ways to find the paths.
/// The first added layer takes precedence.
#[derive(Default)]
pub struct BindgenPaths {
    layers: Vec<Box<dyn BindgenPathsLayer>>,
}

impl BindgenPaths {
    #[cfg(feature = "cargo-metadata")]
    /// Add a layer that finds paths using `cargo metadata`
    ///
    /// Requires the `cargo-metadata` feature.
    pub fn add_cargo_metadata_layer(&mut self, no_deps: bool) -> Result<()> {
        self.add_layer(
            crate::cargo_metadata::CrateConfigSupplier::from_cargo_metadata_command(no_deps)?,
        );
        Ok(())
    }

    /// Add a layer that always uses the same path for config files
    ///
    /// Used to implement the `--config` CLI flag.
    pub fn add_config_override_layer(&mut self, path: Utf8PathBuf) {
        self.add_layer(ConfigOverrideLayer { path })
    }

    /// Add a layer using a [BindgenPathsLayer]
    ///
    /// This can be used to add custom path finding logic.
    pub fn add_layer(&mut self, layer: impl BindgenPathsLayer + 'static) {
        self.layers.push(Box::new(layer));
    }

    /// Get the config table for a crate
    pub fn get_config(&self, crate_name: &str) -> Result<toml::value::Table> {
        for layer in &self.layers {
            if let Some(table) = layer.get_config(crate_name)? {
                return Ok(table);
            }
        }
        Ok(toml::value::Table::default())
    }

    /// Get the UDL path for a crate
    pub fn get_udl_path(&self, crate_name: &str, udl_name: &str) -> Option<Utf8PathBuf> {
        self.layers
            .iter()
            .find_map(|l| l.get_udl_path(crate_name, udl_name))
    }

    /// Get the UDL source for a crate
    pub fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        match self.get_udl_path(crate_name, udl_name) {
            Some(path) => Ok(fs::read_to_string(path)?),
            None => bail!("UDL file {udl_name:?} not found for crate {crate_name:?}"),
        }
    }
}

/// Trait for finding UDL and config paths
pub trait BindgenPathsLayer {
    /// Lookup and load the config TOML for a crate.
    ///
    /// This is usually loaded from `[crate-root]/uniffi.toml`. However, layers are
    /// free to obtain the TOML table for a crate however they want. For example:
    /// * There could be a shared TOML file for all crates and this method could
    ///   return one of the child table values for the specified crate.
    /// * The TOML could be hard-coded into source code and not loaded from a file at all.
    /// * etc.
    fn get_config(&self, _crate_name: &str) -> Result<Option<toml::value::Table>> {
        Ok(None)
    }

    /// Lookup the a UDL file path.
    ///
    /// This is usually the `[crate-root]/src/[udl_name].udl`
    fn get_udl_path(&self, _crate_name: &str, _udl_name: &str) -> Option<Utf8PathBuf> {
        None
    }
}

struct ConfigOverrideLayer {
    path: Utf8PathBuf,
}

impl BindgenPathsLayer for ConfigOverrideLayer {
    fn get_config(&self, _crate_name: &str) -> Result<Option<toml::value::Table>> {
        // ConfigOverrideLayer is used when an expected config override is expected to exist
        // (eg, --config args), so the file not existing should be an error.
        let contents = fs::read_to_string(&self.path)
            .with_context(|| format!("read file: {:?}", self.path))?;
        let toml = toml::de::from_str(&contents)
            .with_context(|| format!("parse toml: {:?}", self.path))?;
        Ok(Some(toml))
    }
}
