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

use crate::{BindingGenerator, Component, GenerationSettings};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use fs_err as fs;
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

pub struct SwiftBindingGenerator;
impl BindingGenerator for SwiftBindingGenerator {
    type Config = Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("swift")) {
                Some(v) => v.clone().try_into()?,
                None => Default::default(),
            },
        )
    }

    fn update_component_configs(
        &self,
        _settings: &GenerationSettings,
        components: &mut Vec<Component<Self::Config>>,
    ) -> Result<()> {
        for c in &mut *components {
            c.config
                .module_name
                .get_or_insert_with(|| c.ci.namespace().into());
        }
        Ok(())
    }

    /// Unlike other target languages, binding to Rust code from Swift involves more than just
    /// generating a `.swift` file. We also need to produce a `.h` file with the C-level API
    /// declarations, and a `.modulemap` file to tell Swift how to use it.
    fn write_bindings(
        &self,
        settings: &GenerationSettings,
        components: &[Component<Self::Config>],
    ) -> Result<()> {
        for Component { ci, config, .. } in components {
            let Bindings {
                header,
                library,
                modulemap,
            } = generate_bindings(config, ci)?;

            let source_file = settings
                .out_dir
                .join(format!("{}.swift", config.module_name()));
            fs::write(&source_file, library)?;

            let header_file = settings.out_dir.join(config.header_filename());
            fs::write(header_file, header)?;

            if let Some(modulemap) = modulemap {
                let modulemap_file = settings.out_dir.join(config.modulemap_filename());
                fs::write(modulemap_file, modulemap)?;
            }

            if settings.try_format_code {
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

        Ok(())
    }
}

/// Generate Swift bindings
///
/// This is used by the uniffi-bindgen-swift command, which supports Swift-specific options.
///
/// In the future, we may want to replace the generalized `uniffi-bindgen` with a set of
/// specialized `uniffi-bindgen-[language]` commands.
pub fn generate_swift_bindings(options: SwiftBindingsOptions) -> Result<()> {
    #[cfg(feature = "cargo-metadata")]
    let config_supplier = {
        use crate::cargo_metadata::CrateConfigSupplier;
        let mut cmd = cargo_metadata::MetadataCommand::new();
        if options.metadata_no_deps {
            cmd.no_deps();
        }
        let metadata = cmd.exec().context("error running cargo metadata")?;
        CrateConfigSupplier::from(metadata)
    };
    #[cfg(not(feature = "cargo-metadata"))]
    let config_supplier = crate::EmptyCrateConfigSupplier;

    fs::create_dir_all(&options.out_dir)?;

    let mut components =
        crate::library_mode::find_components(&options.library_path, &config_supplier)?
            // map the TOML configs into a our Config struct
            .into_iter()
            .map(|Component { ci, config }| {
                let config = SwiftBindingGenerator.new_config(&config.into())?;
                Ok(Component { ci, config })
            })
            .collect::<Result<Vec<_>>>()?;
    SwiftBindingGenerator
        .update_component_configs(&GenerationSettings::default(), &mut components)?;

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

    // find the library name by stripping the extension and leading `lib` from the library path
    let library_name = {
        let stem = options
            .library_path
            .file_stem()
            .with_context(|| format!("Invalid library path {}", options.library_path))?;
        match stem.strip_prefix("lib") {
            Some(name) => name,
            None => stem,
        }
    };

    let module_name = options
        .module_name
        .unwrap_or_else(|| library_name.to_string());
    let modulemap_filename = options
        .modulemap_filename
        .unwrap_or_else(|| format!("{library_name}.modulemap"));

    if options.generate_modulemap {
        let mut header_filenames: Vec<_> = components
            .iter()
            .map(|Component { config, .. }| config.header_filename())
            .collect();
        header_filenames.sort();
        let modulemap_source =
            generate_modulemap(module_name, header_filenames, options.xcframework)?;
        let modulemap_path = options.out_dir.join(modulemap_filename);
        fs::write(modulemap_path, modulemap_source)?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct SwiftBindingsOptions {
    pub generate_swift_sources: bool,
    pub generate_headers: bool,
    pub generate_modulemap: bool,
    pub library_path: Utf8PathBuf,
    pub out_dir: Utf8PathBuf,
    pub xcframework: bool,
    pub module_name: Option<String>,
    pub modulemap_filename: Option<String>,
    pub metadata_no_deps: bool,
}
