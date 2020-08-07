/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};
use std::{env, process::Command};

pub fn generate_scaffolding(idl_file: &str) -> Result<()> {
    println!("cargo:rerun-if-changed={}", idl_file);
    // Why don't we just depend on uniffi-bindgen and call the public functions?
    // Calling the command line helps making sure that the generated swift/Kotlin/whatever
    // bindings were generated with the same version of uniffi as the Rust scaffolding code.
    let out_dir = env::var("OUT_DIR").map_err(|_| anyhow::anyhow!("$OUT_DIR missing?!"))?;
    if Command::new("uniffi-bindgen").output().is_err() {
        bail!("It looks like uniffi-bindgen is not installed. You can do so by running `cargo install uniffi-bindgen`")
    }
    let status = Command::new("uniffi-bindgen")
        .args(&["scaffolding", "--out-dir", &out_dir, idl_file])
        .status()?;
    if !status.success() {
        bail!("Error while generating scaffolding code");
    }
    Ok(())
}
