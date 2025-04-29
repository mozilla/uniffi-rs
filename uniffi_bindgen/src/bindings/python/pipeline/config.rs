/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use serde::Deserialize;

pub fn pass(module: &mut Module) -> Result<()> {
    let config = match &module.config_toml {
        Some(toml) => toml::from_str(toml)?,
        None => Config::default(),
    };
    let mut config = config.bindings.python;
    module.visit_mut(|custom: &mut CustomType| {
        custom.config = config.custom_types.shift_remove(&custom.name);
    });
    module.config = config;
    Ok(())
}

// These structs exist so that we can easily deserialize the entire `uniffi.toml` file.
// We then extract the `PythonConfig`, which is what we actually care about.

#[derive(Debug, Clone, Deserialize, Node)]
pub struct Config {
    #[serde(default)]
    bindings: BindingsConfig,
}

#[derive(Debug, Clone, Deserialize, Node)]
pub struct BindingsConfig {
    #[serde(default)]
    python: PythonConfig,
}
