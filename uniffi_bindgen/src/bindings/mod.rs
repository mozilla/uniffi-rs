/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

use std::fs;

use anyhow::Result;
use camino::Utf8PathBuf;

use crate::{BindgenLoader, BindgenPaths};
mod kotlin;
pub mod python;
mod ruby;
mod swift;
pub use swift::{generate_swift_bindings, SwiftBindingsOptions};

#[cfg(feature = "bindgen-tests")]
pub use self::{
    kotlin::test as kotlin_test, python::test as python_test, ruby::test as ruby_test,
    swift::test as swift_test,
};

#[cfg(feature = "bindgen-tests")]
/// Mode for the `run_script` function defined for each language
#[derive(Clone, Debug)]
pub struct RunScriptOptions {
    pub show_compiler_messages: bool,
}

#[cfg(feature = "bindgen-tests")]
impl Default for RunScriptOptions {
    fn default() -> Self {
        Self {
            show_compiler_messages: true,
        }
    }
}

/// Generate bindings
///
/// This implements the default uniffi-bindgen command
pub fn generate(options: GenerateOptions) -> Result<()> {
    generate_with_bindgen_paths(options, BindgenPaths::default())
}

/// Generate bindings with custom bindgen paths.
pub fn generate_with_bindgen_paths(
    options: GenerateOptions,
    mut paths: BindgenPaths,
) -> Result<()> {
    if let Some(path) = &options.config_override {
        paths.add_config_override_layer(path.clone());
    }

    #[cfg(feature = "cargo-metadata")]
    paths.add_cargo_metadata_layer(crate::CargoMetadataOptions {
        no_deps: options.metadata_no_deps,
        target: options.target.clone(),
        all_features: options.all_features,
        no_default_features: options.no_default_features,
        features: options.features.clone(),
    })?;

    fs::create_dir_all(&options.out_dir)?;

    let loader = BindgenLoader::new(paths);
    for language in options.languages.iter() {
        match language {
            TargetLanguage::Swift => {
                swift::generate(&loader, options.clone())?;
            }
            TargetLanguage::Kotlin => {
                kotlin::generate(&loader, options.clone())?;
            }
            TargetLanguage::Python => {
                python::generate(&loader, options.clone())?;
            }
            TargetLanguage::Ruby => {
                ruby::generate(&loader, options.clone())?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Default)]
pub struct GenerateOptions {
    /// Languages to generate bindings for
    pub languages: Vec<TargetLanguage>,
    /// Path to the UDL, library file, or "src" to generate using Rust sources
    pub source: Utf8PathBuf,
    /// Features to enable when generating from Rust sources
    pub features: Vec<String>,
    /// Enable all features
    pub all_features: bool,
    /// Don't auto-enable default features
    pub no_default_features: bool,
    /// Target triple to use when generating from Rust sources
    pub target: Option<String>,
    /// Directory to write generated files.
    pub out_dir: Utf8PathBuf,
    /// Path to the config file to use, if None bindings generators will load
    /// `[crate-root]/uniffi.toml`
    pub config_override: Option<Utf8PathBuf>,
    /// Run the generated code through a source code formatter
    pub format: bool,
    /// Limit binding generate to a single crate
    pub crate_filter: Option<String>,
    /// Exclude dependencies when running "cargo metadata".
    /// This will mean external types may not be resolved if they are implemented in crates
    /// outside of this workspace.
    /// This can be used in environments when all types are in the namespace and fetching
    /// all sub-dependencies causes obscure platform specific problems.
    pub metadata_no_deps: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum TargetLanguage {
    Kotlin,
    Python,
    Ruby,
    Swift,
}
