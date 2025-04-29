/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;
use fs_err as fs;
use serde::{Deserialize, Serialize};

mod pipeline;
pub use pipeline::pipeline;

#[cfg(feature = "bindgen-tests")]
pub mod test;

// Config options to customize the generated python.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) cdylib_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTypeConfig {
    // This `CustomTypeConfig` doesn't have a `type_name` like the others -- which is why we have
    // separate structs rather than a shared one.
    imports: Option<Vec<String>>,
    into_custom: String, // b/w compat alias for lift
    lift: String,
    from_custom: String, // b/w compat alias for lower
    lower: String,
}

pub fn run_pipeline(initial_root: pipeline::initial::Root, out_dir: &Utf8Path) -> Result<()> {
    let python_root = pipeline().execute(initial_root)?;
    println!("writing out {out_dir}");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }
    for module in python_root.modules.values() {
        let path = out_dir.join(format!("{}.py", module.name));
        let content = module.render()?;
        println!("writing {path}");
        fs::write(path, content)?;
    }
    Ok(())
}
