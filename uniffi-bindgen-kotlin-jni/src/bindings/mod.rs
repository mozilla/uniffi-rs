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
            return render_uniffi_package(&root, out_dir);
        }
        Some(crate_filter) => {
            // Only generate bindings for the crate
            let crate_filter = crate_filter.replace("-", "_");
            root.packages.retain(|pkg| pkg.crate_name == crate_filter);
        }
        None => {
            // Render the shared package and don't filter
            render_uniffi_package(&root, out_dir)?;
        }
    }

    // Create a package for each crate
    let java_8_compatible = root.java_8_compatible();
    for package in root.packages {
        if package.is_empty() {
            continue;
        }
        let package_dir = out_dir.join(package.name.split(".").collect::<Utf8PathBuf>());
        render(
            &package_dir.join("uniffi"),
            PackageFfi { package: &package },
        )?;

        render(
            &package_dir,
            BindingsPackage {
                package,
                java_8_compatible,
            },
        )?;
    }

    Ok(())
}

pub fn render_uniffi_package(root: &Root, out_dir: &Utf8Path) -> Result<()> {
    render(
        &out_dir.join("uniffi"),
        UniffiShared {
            cdylib: root.cdylib_name()?,
            root,
        },
    )
}

pub fn render<T: Template>(package_dir: &Utf8Path, template: T) -> Result<()> {
    if !package_dir.exists() {
        std::fs::create_dir_all(package_dir)?;
    }
    let output = template.render()?;
    let mut f = File::create(package_dir.join("Uniffi.kt"))?;
    write!(f, "{output}")?;
    Ok(())
}

#[derive(Debug, Clone, Template)]
#[template(syntax = "kt", escape = "none", path = "Package.kt")]
pub struct BindingsPackage {
    package: Package,
    java_8_compatible: bool,
}

/// `[package].uniffi` -- Package-specific FFI code
///
/// This contains functions to lift/lower types defined in that package.
///
/// The package name in `[package_name].uniffi`.
#[derive(Debug, Clone, Template)]
#[template(syntax = "kt", escape = "none", path = "ffi/Package.kt")]
pub struct PackageFfi<'a> {
    package: &'a Package,
}

/// `uniffi` package: shared FFI code.
///
/// This contains functions to lift/lower types builtin UniFFI types,
/// including composite types like `Vec<Option<String>>`
#[derive(Debug, Clone, Template)]
#[template(syntax = "kt", escape = "none", path = "ffi/Shared.kt")]
pub struct UniffiShared<'a> {
    cdylib: String,
    root: &'a Root,
}

// Defining filters modules to make template errors a bit better
mod filters {}
