/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{fs::File, io::Write};

use anyhow::Result;
use askama::Template;
use camino::{Utf8Path, Utf8PathBuf};

pub use crate::pipeline::*;

pub fn generate(
    initial_root: InitialRoot,
    out_dir: &Utf8Path,
    crate_filter: Option<String>,
) -> Result<()> {
    let mut root = pipeline().execute(initial_root)?;
    if let Some(crate_filter) = crate_filter {
        let crate_filter = crate_filter.replace("-", "_");
        root.packages.retain(|pkg| pkg.name == crate_filter);
    }
    render(
        &out_dir.join("uniffi"),
        "Uniffi.kt",
        UniffiPackage {
            cdylib: root.cdylib_name()?,
            root: &root,
        },
    )?;

    // Create a package for each crate
    for package in root.packages {
        let package_dir = out_dir.join(package.name.split(".").collect::<Utf8PathBuf>());
        let filename = format!("{}.kt", package.name);
        render(&package_dir, &filename, BindingsPackage { package })?;
    }

    Ok(())
}

pub fn render<T: Template>(package_dir: &Utf8Path, filename: &str, template: T) -> Result<()> {
    if !package_dir.exists() {
        std::fs::create_dir_all(package_dir)?;
    }
    let output = template.render()?;
    let mut f = File::create(package_dir.join(filename))?;
    write!(f, "{output}")?;
    Ok(())
}

#[derive(Debug, Clone, Template)]
#[template(syntax = "kt", escape = "none", path = "Package.kt")]
pub struct BindingsPackage {
    package: Package,
}

#[derive(Debug, Clone, Template)]
#[template(syntax = "kt", escape = "none", path = "shared/Package.kt")]
pub struct UniffiPackage<'a> {
    cdylib: String,
    root: &'a Root,
}
