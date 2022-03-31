/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Swift bindings backend for UniFFI
//!
//! This module generates Swift bindings from a [`ComponentInterface`] definition,
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
//! Most of the concepts in a [`ComponentInterface`] have an obvious counterpart in Swift,
//! with the details documented in inline comments where appropriate.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use uniffi_bindgen::{generate_external_bindings, BindingGenerator, ComponentInterface};

mod gen_swift;

/// The Swift bindings generated from a [`ComponentInterface`].
pub struct Bindings {
    /// The contents of the generated `.swift` file, as a string.
    library: String,
    /// The contents of the generated `.h` file, as a string.
    header: String,
    /// The contents of the generated `.modulemap` file, as a string.
    modulemap: Option<String>,
}

struct SwiftBindingGenerator {
    try_format_code: bool,
}

impl SwiftBindingGenerator {
    pub fn new(try_format_code: bool) -> Self {
        Self { try_format_code }
    }
}

impl BindingGenerator for SwiftBindingGenerator {
    type Config = gen_swift::Config;

    fn write_bindings(
        &self,
        ci: ComponentInterface,
        config: Self::Config,
        out_dir: &Path,
    ) -> anyhow::Result<()> {
        let out_path = PathBuf::from(out_dir);

        let Bindings {
            header,
            library,
            modulemap,
        } = gen_swift::generate_bindings(&config, &ci)?;

        let mut source_file = out_path.clone();
        source_file.push(format!("{}.swift", config.module_name));
        let mut l =
            File::create(&source_file).context("Failed to create .swift file for bindings")?;
        write!(l, "{}", library)?;

        let mut header_file = out_path.clone();
        header_file.push(config.header_filename());
        let mut h = File::create(&header_file).context("Failed to create .h file for bindings")?;
        write!(h, "{}", header)?;

        if let Some(modulemap) = modulemap {
            let mut modulemap_file = out_path;
            modulemap_file.push(config.modulemap_filename());
            let mut m = File::create(&modulemap_file)
                .context("Failed to create .modulemap file for bindings")?;
            write!(m, "{}", modulemap)?;
        }

        if self.try_format_code {
            if let Err(e) = Command::new("swiftformat")
                .arg(source_file.to_str().unwrap())
                .output()
            {
                println!(
                    "Warning: Unable to auto-format {} using swiftformat: {:?}",
                    source_file.file_name().unwrap().to_str().unwrap(),
                    e
                )
            }
        }

        Ok(())
    }
}

pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = clap::App::new("uniffi-bindgen-swift")
        .about("Scaffolding and bindings generator for Rust")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("no_format")
            .long("--no-format")
            .help("Do not format the generated code with rustfmt (useful for maintainers)"),
        )
        .arg(
            clap::Arg::with_name("out_dir")
                .long("--out-dir")
                .short("-o")
                .takes_value(true)
                .help("Directory in which to write generated files. Default is same folder as .udl file."),
        )
        .arg(clap::Arg::with_name("udl_file").required(true))
        .arg(
            clap::Arg::with_name("config")
            .long("--config-path")
            .takes_value(true)
            .help("Path to the optional uniffi config file. If not provided, uniffi-bindgen will try to guess it from the UDL's file location.")
        )
        .get_matches_from(args);

    let binding_generator = SwiftBindingGenerator::new(!matches.is_present("no_format"));
    generate_external_bindings(
        binding_generator,
        matches.value_of_os("udl_file").unwrap(), // Required
        matches.value_of_os("config"),
        matches.value_of_os("out_dir"),
    )
}

pub fn run_main() -> Result<()> {
    run(std::env::args_os())
}
