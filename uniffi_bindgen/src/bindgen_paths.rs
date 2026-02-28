/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;

use anyhow::bail;
use camino::Utf8PathBuf;

use crate::Result;

/// Responsible for looking up crate root paths, optionally overriding paths to UDL, crate configs, etc
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
    pub fn add_cargo_metadata_layer(&mut self, options: CargoMetadataOptions) -> Result<()> {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if options.no_deps {
            cmd.no_deps();
        }
        self.add_layer(
            crate::cargo_metadata::CrateConfigSupplier::from_cargo_metadata_command(cmd, options)?,
        );
        Ok(())
    }

    /// Add a layer using a [BindgenPathsLayer]
    ///
    /// This can be used to add custom path finding logic.
    pub fn add_layer(&mut self, layer: impl BindgenPathsLayer + 'static) {
        self.layers.push(Box::new(layer));
    }

    /// Get the root directory for a crate
    pub fn get_crate_root(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.layers
            .iter()
            .find_map(|l| l.get_crate_root(crate_name))
    }

    /// Get the config file path for a crate
    pub fn get_config_path(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.layers
            .iter()
            .find_map(|l| l.get_config_path(crate_name))
    }

    /// Get the UDL path for a crate
    pub fn get_udl_path(&self, crate_name: &str, udl_name: &str) -> Option<Utf8PathBuf> {
        self.layers
            .iter()
            .find_map(|l| l.get_udl_path(crate_name, udl_name))
    }

    /// Get the name of the cdylib for a crate
    pub fn get_cdylib_name(&self, crate_name: &str) -> Option<String> {
        self.layers
            .iter()
            .find_map(|l| l.get_cdylib_name(crate_name))
    }

    /// Get the UDL source for a crate
    pub fn get_udl(&self, crate_name: &str, udl_name: &str) -> Result<String> {
        match self.get_udl_path(crate_name, udl_name) {
            Some(path) => Ok(fs::read_to_string(path)?),
            None => bail!("UDL file {udl_name:?} not found for crate {crate_name:?}"),
        }
    }

    /// Get crate sources that should be parsed
    ///
    /// Given a source crate, this returns [SourceCrate] info for crates that:
    /// * Are dependencies of `source_crate` either directly or indirectly
    /// * Have a direct dependency on `uniffi`.
    ///
    /// This list includes the source crate itself.
    pub fn get_source_crates(&self, source_crate: &str) -> Option<Vec<SourceCrate>> {
        self.layers
            .iter()
            .find_map(|l| l.get_source_crates(source_crate))
    }
}

#[cfg(feature = "cargo-metadata")]
#[derive(Debug, Clone, Default)]
pub struct CargoMetadataOptions {
    pub no_deps: bool,
    pub no_default_features: bool,
    pub all_features: bool,
    pub features: Vec<String>,
}

#[derive(Debug)]
pub struct SourceCrate {
    pub name: String,
    pub src_path: Utf8PathBuf,
    pub features: Vec<String>,
}

/// Trait for finding crate roots and UDL paths.
///
/// Implement `get_crate_root` to provide crate discovery. The other methods
/// have default implementations that derive from the crate root.
pub trait BindgenPathsLayer {
    /// Find the root directory of a crate.
    fn get_crate_root(&self, _crate_name: &str) -> Option<Utf8PathBuf> {
        None
    }

    /// Find the config file path for a crate.
    ///
    /// Default implementation returns `{crate_root}/uniffi.toml`.
    fn get_config_path(&self, crate_name: &str) -> Option<Utf8PathBuf> {
        self.get_crate_root(crate_name)
            .map(|root| root.join("uniffi.toml"))
    }

    /// Lookup a UDL file path.
    ///
    /// Default implementation returns `{crate_root}/src/{udl_name}.udl`.
    fn get_udl_path(&self, crate_name: &str, udl_name: &str) -> Option<Utf8PathBuf> {
        self.get_crate_root(crate_name)
            .map(|root| root.join("src").join(format!("{udl_name}.udl")))
    }

    /// Get the name of the cdylib for a crate
    fn get_cdylib_name(&self, _source_crate: &str) -> Option<String> {
        None
    }

    /// Get a list of default crate names
    ///
    /// This is used when the `uniffi_bindgen` source is a `Cargo.toml` file
    /// and no single crate is specified with `--crate`.
    fn get_source_crates(&self, _source_crate: &str) -> Option<Vec<SourceCrate>> {
        None
    }
}
