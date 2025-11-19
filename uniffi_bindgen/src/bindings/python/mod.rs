/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;
use fs_err as fs;

use crate::{bindings::GenerateOptions, BindgenLoader};

pub mod filters;
mod pipeline;
pub use pipeline::pipeline;

#[cfg(feature = "bindgen-tests")]
pub mod test;

/// Generate Python bindings
pub fn generate(loader: &BindgenLoader, options: GenerateOptions) -> Result<()> {
    let metadata = loader.load_metadata(&options.source)?;
    let root = loader.load_pipeline_initial_root(&options.source, metadata)?;
    run_pipeline(root, &options.out_dir)?;

    Ok(())
}

pub fn run_pipeline(initial_root: pipeline::initial::Root, out_dir: &Utf8Path) -> Result<()> {
    let python_root = pipeline().execute(initial_root)?;
    println!("writing out {out_dir}");
    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }
    for module in python_root.namespaces.values() {
        let path = out_dir.join(format!("{}.py", module.name));
        let content = module.render()?;
        println!("writing {path}");
        fs::write(path, content)?;
    }
    Ok(())
}
