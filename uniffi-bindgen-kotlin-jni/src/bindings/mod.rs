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
    match crate_filter.as_deref() {
        Some("uniffi") => {
            // special-case: only render the shared package
            return render_shared_package(&root, out_dir);
        }
        Some(crate_filter) => {
            // Only generate bindings for the crate
            let crate_filter = crate_filter.replace("-", "_");
            root.packages.retain(|pkg| pkg.crate_name == crate_filter);
        }
        None => {
            // Render the shared package and don't filter
            render_shared_package(&root, out_dir)?;
        }
    }

    // Create a package for each crate
    for package in root.packages {
        let package_dir = out_dir.join(package.name.split(".").collect::<Utf8PathBuf>());
        let filename = format!("{}.kt", package.name);
        render(&package_dir, &filename, BindingsPackage { package })?;
    }

    Ok(())
}

pub fn render_shared_package(root: &Root, out_dir: &Utf8Path) -> Result<()> {
    render(
        &out_dir.join("uniffi"),
        "Uniffi.kt",
        UniffiPackage {
            cdylib: root.cdylib_name()?,
            root,
        },
    )
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

// Defining filters modules to make template errors a bit better
mod filters {}
