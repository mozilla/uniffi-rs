/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{fs::File, io::Write};

use anyhow::Result;
use askama::Template;
use camino::Utf8Path;

use crate::pipeline::*;

pub fn generate(initial_root: InitialRoot, out_dir: &Utf8Path) -> Result<()> {
    let root = pipeline().execute(initial_root)?;
    let scaffolding = Scaffolding { root };
    let output = scaffolding.render()?;
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)?;
    }
    let out_path = out_dir.join("uniffi_bindgen_kotlin_jni.uniffi.rs");
    let mut f = File::create(out_path)?;
    write!(f, "{output}")?;

    Ok(())
}

#[derive(Debug, Clone, Template)]
#[template(syntax = "rs", escape = "none", path = "scaffolding.rs")]
pub struct Scaffolding {
    root: Root,
}

// Defining filters modules to make templates errors a bit better
mod filters {}
