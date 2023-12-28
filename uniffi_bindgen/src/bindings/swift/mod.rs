/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
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
//!
//! To handle lifting/lowering/serializing types across the FFI boundary, the Swift code
//! defines a `protocol ViaFfi` that is analogous to the `uniffi::ViaFfi` Rust trait.
//! Each type that can traverse the FFI conforms to the `ViaFfi` protocol, which specifies:
//!
//!  * The corresponding low-level type.
//!  * How to lift from and lower into into that type.
//!  * How to read from and write into a byte buffer.
//!

use std::process::Command;

use anyhow::Result;
use camino::Utf8Path;
use fs_err as fs;

pub mod gen_swift;
pub use gen_swift::{generate_bindings, generate_mocks, Config};
mod test;

use super::super::interface::ComponentInterface;
pub use test::{run_script, run_test};

/// The Swift bindings generated from a [`ComponentInterface`].
///
pub struct Bindings {
    /// The contents of the generated `.swift` file, as a string.
    library: String,
    /// The contents of the generated `.h` file, as a string.
    header: String,
    /// The contents of the generated `.modulemap` file, as a string.
    modulemap: Option<String>,
}

/// Write UniFFI component bindings for Swift as files on disk.
///
/// Unlike other target languages, binding to Rust code from Swift involves more than just
/// generating a `.swift` file. We also need to produce a `.h` file with the C-level API
/// declarations, and a `.modulemap` file to tell Swift how to use it.
pub fn write_bindings(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Utf8Path,
    try_format_code: bool,
) -> Result<()> {
    let Bindings {
        header,
        library,
        modulemap,
    } = generate_bindings(config, ci)?;

    let source_file = out_dir.join(format!("{}.swift", config.module_name()));
    fs::write(&source_file, library)?;

    let header_file = out_dir.join(config.header_filename());
    fs::write(header_file, header)?;

    if let Some(modulemap) = modulemap {
        let modulemap_file = out_dir.join(config.modulemap_filename());
        fs::write(modulemap_file, modulemap)?;
    }

    if try_format_code {
        format_code(&source_file)
    }

    Ok(())
}

/// Write UniFFI mocks for Swift as files on disk.
pub fn write_mocks(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Utf8Path,
    try_format_code: bool,
) -> Result<()> {
    let mocks = generate_mocks(config, ci)?;

    let source_file = out_dir.join(format!("{}_mocks.swift", config.module_name()));
    fs::write(&source_file, mocks)?;

    if try_format_code {
        format_code(&source_file)
    }

    Ok(())
}

fn format_code(source_file: &Utf8Path) {
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
