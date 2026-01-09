/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Swift bindings backend for UniFFI
//!
//! This module generates Swift bindings from a [`crate::ComponentInterface`] definition,
//! using Swift's builtin support for loading C header files.
//!
//! Conceptually, the generated bindings are split into two Swift modules, one for the low-level
//! C FFI layer and one for the higher-level Swift bindings. For a UniFFI component named "example"
//! we generate:
//!
//!   * A C header file `exampleFFI.h` declaring the low-level structs and functions for calling
//!     into Rust, along with a corresponding `exampleFFI.modulemap` to expose them to Swift.
//!
//!   * A Swift source file `example.swift` that imports the `exampleFFI` module and wraps it
//!     to provide the higher-level Swift API.
//!
//! Most of the concepts in a [`crate::ComponentInterface`] have an obvious counterpart in Swift,
//! with the details documented in inline comments where appropriate.
//!
//! To handle lifting/lowering/serializing types across the FFI boundary, the Swift code
//! defines a `protocol ViaFfi` that is analogous to the `uniffi::ViaFfi` Rust trait.
//! Each type that can traverse the FFI conforms to the `ViaFfi` protocol, which specifies:
//!
//!  * The corresponding low-level type.
//!  * How to lift from and lower into into that type.
//!  * How to read from and write into a byte buffer.
//!

use crate::{
    bindings::GenerateOptions, interface::rename, BindgenLoader, BindgenPaths, Component,
    ComponentInterface,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use fs_err as fs;
use std::collections::HashMap;
use std::process::Command;

mod gen_swift;
use gen_swift::{generate_bindings, generate_header, generate_modulemap, generate_swift, Config};

#[cfg(feature = "bindgen-tests")]
pub mod test;

/// The Swift bindings generated from a [`crate::ComponentInterface`].
///
struct Bindings {
    /// The contents of the generated `.swift` file, as a string.
    library: String,
    /// The contents of the generated `.h` file, as a string.
    header: String,
    /// The contents of the generated `.modulemap` file, as a string.
    modulemap: Option<String>,
}

/// Generate Swift bindings
///
/// Returns the components generated
pub fn generate(
    loader: &BindgenLoader,
    options: GenerateOptions,
) -> Result<Vec<Component<Config>>> {
    let metadata = loader.load_metadata(&options.source)?;
    let cis = loader.load_cis(metadata)?;
    let mut components = loader.load_components(cis, parse_config)?;
    apply_renames(&mut components);
    for c in components.iter_mut() {
        // Call derive_ffi_functions after `apply_renames`
        c.ci.derive_ffi_funcs()?;
    }

    for Component { ci, config, .. } in components.iter_mut() {
        if let Some(crate_filter) = &options.crate_filter {
            if ci.crate_name() != crate_filter {
                continue;
            }
        }
        let Bindings {
            header,
            library,
            modulemap,
        } = generate_bindings(config, ci)?;

        let source_file = options
            .out_dir
            .join(format!("{}.swift", config.module_name()));
        fs::write(&source_file, library)?;

        let header_file = options.out_dir.join(config.header_filename());
        fs::write(header_file, header)?;

        if let Some(modulemap) = modulemap {
            let modulemap_file = options.out_dir.join(config.modulemap_filename());
            fs::write(modulemap_file, modulemap)?;
        }

        if options.format {
            let commands_to_try = [
                // Available in Xcode 16.
                vec!["xcrun", "swift-format"],
                // The official swift-format command name.
                vec!["swift-format"],
                // Shortcut for the swift-format command.
                vec!["swift", "format"],
                vec!["swiftformat"],
            ];

            let successful_output = commands_to_try.into_iter().find_map(|command| {
                Command::new(command[0])
                    .args(&command[1..])
                    .arg(source_file.as_str())
                    .output()
                    .ok()
            });
            if successful_output.is_none() {
                println!(
                    "Warning: Unable to auto-format {} using swift-format. Please make sure it is installed.",
                    source_file.as_str()
                );
            }
        }
    }
    Ok(components)
}

/// Generate Swift bindings (specialized version)
///
/// This is used by the uniffi-bindgen-swift command, which supports Swift-specific options.
///
/// In the future, we may want to replace the generalized `uniffi-bindgen` with a set of
/// specialized `uniffi-bindgen-[language]` commands.
pub fn generate_swift_bindings(options: SwiftBindingsOptions) -> Result<()> {
    #[cfg(not(feature = "cargo-metadata"))]
    let paths = BindgenPaths::default();

    #[cfg(feature = "cargo-metadata")]
    let mut paths = BindgenPaths::default();

    #[cfg(feature = "cargo-metadata")]
    paths.add_cargo_metadata_layer(options.metadata_no_deps)?;

    fs::create_dir_all(&options.out_dir)?;

    let loader = BindgenLoader::new(paths);
    let metadata = loader.load_metadata(&options.source)?;
    let cis = loader.load_cis(metadata)?;
    let mut components = loader.load_components(cis, parse_config)?;
    apply_renames(&mut components);
    // Call derive_ffi_funcs after apply_renames()
    for Component { ci, .. } in components.iter_mut() {
        ci.derive_ffi_funcs()?;
    }

    for Component { ci, config } in &components {
        if options.generate_swift_sources {
            let source_file = options
                .out_dir
                .join(format!("{}.swift", config.module_name()));
            fs::write(&source_file, generate_swift(config, ci)?)?;
        }

        if options.generate_headers {
            let header_file = options.out_dir.join(config.header_filename());
            fs::write(header_file, generate_header(config, ci)?)?;
        }
    }

    // Derive the default module_name/modulemap_filename from the source filename.
    let source_basename = loader.source_basename(&options.source);

    let module_name = options
        .module_name
        .unwrap_or_else(|| source_basename.to_string());
    let modulemap_filename = options
        .modulemap_filename
        .unwrap_or_else(|| format!("{source_basename}.modulemap"));

    if options.generate_modulemap {
        let mut header_filenames: Vec<_> = components
            .iter()
            .map(|Component { config, .. }| config.header_filename())
            .collect();
        header_filenames.sort();
        let modulemap_source = generate_modulemap(
            module_name,
            header_filenames,
            options.xcframework,
            options.link_frameworks,
        )?;
        let modulemap_path = options.out_dir.join(modulemap_filename);
        fs::write(modulemap_path, modulemap_source)?;
    }

    Ok(())
}

fn parse_config(ci: &ComponentInterface, root_toml: toml::Value) -> Result<Config> {
    let mut config: Config = match root_toml.get("bindings").and_then(|b| b.get("swift")) {
        Some(v) => v.clone().try_into()?,
        None => Default::default(),
    };
    config
        .module_name
        .get_or_insert_with(|| ci.namespace().into());
    Ok(config)
}

#[derive(Debug, Default)]
pub struct SwiftBindingsOptions {
    pub generate_swift_sources: bool,
    pub generate_headers: bool,
    pub generate_modulemap: bool,
    pub source: Utf8PathBuf,
    pub out_dir: Utf8PathBuf,
    pub xcframework: bool,
    pub module_name: Option<String>,
    pub modulemap_filename: Option<String>,
    pub metadata_no_deps: bool,
    pub link_frameworks: Vec<String>,
}

// A helper for renaming items.
fn apply_renames(components: &mut Vec<Component<Config>>) {
    let mut module_renames = HashMap::new();
    // Collect all rename configurations from all components, keyed by module_path
    for c in components.iter() {
        if !c.config.rename.is_empty() {
            let module_path = c.ci.crate_name().to_string();
            module_renames.insert(module_path, c.config.rename.clone());
        }
    }

    // Apply rename configurations to all components
    if !module_renames.is_empty() {
        for c in &mut *components {
            rename(&mut c.ci, &module_renames);
        }
    }
}
