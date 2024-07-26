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
//!    into Rust, along with a corresponding `exampleFFI.modulemap` to expose them to Swift.
//!
//!   * A Swift source file `example.swift` that imports the `exampleFFI` module and wraps it
//!    to provide the higher-level Swift API.
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
use anyhow::Result;
use fs_err as fs;
use std::process::Command;
use crate::interface::LanguageComponentInterface;

mod gen_swift;
use gen_swift::{generate_bindings, Config};
mod test;
pub use test::{run_script, run_test};

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
                if let Err(e) = Command::new("swiftformat")
                    .arg(source_file.as_str())
                    .output()
                {
                    println!(
                        "Warning: Unable to auto-format {} using swiftformat: {e:?}",
                        source_file.file_name().unwrap(),
                    );
                }
            }
        }

        Ok(())
    }
}
