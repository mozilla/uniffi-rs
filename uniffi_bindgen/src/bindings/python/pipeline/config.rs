/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// This module contains the serde structs to parse the `uniffi.toml` config.
use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;

use uniffi_pipeline::Node;

// These just exist so we can parse the entire `uniffi.toml` file, the codegen only uses the
// `PythonConfig` part.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub bindings: BindingsConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BindingsConfig {
    #[serde(default)]
    pub python: PythonConfig,
}

// Config options to customize the generated python.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PythonConfig {
    pub(super) cdylib_name: Option<String>,
    #[serde(default)]
    pub custom_types: IndexMap<String, CustomTypeConfig>,
    #[serde(default)]
    pub external_packages: IndexMap<String, String>,
}

#[derive(Debug, Clone, Node, Default, Deserialize)]
#[serde(default)]
pub struct CustomTypeConfig {
    pub imports: Option<Vec<String>>,
    pub type_name: Option<String>, // b/w compat alias for lift
    pub into_custom: String,       // b/w compat alias for lift
    pub lift: String,
    pub from_custom: String, // b/w compat alias for lower
    pub lower: String,
}

impl PythonConfig {
    pub fn from_uniffi_toml(toml: &str) -> Result<Self> {
        let root: Config = toml::from_str(toml)?;
        Ok(root.bindings.python)
    }
}
