/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};

pub mod gen_rust;
pub use gen_rust::{Config, RustWrapper};

use super::super::interface::ComponentInterface;

// Generate rust bindings for the given ComponentInterface, in the given output directory.

pub fn write_bindings(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Path,
    try_format_code: bool,
) -> Result<()> {
    let mut rs_file = PathBuf::from(out_dir);
    rs_file.push(format!("{}_uniffi.rs", ci.namespace()));
    let mut f = File::create(&rs_file).context("Failed to create .rs file for bindings")?;
    write!(f, "{}", generate_rust_bindings(config, ci)?)?;

    if try_format_code {
        if let Err(e) = Command::new("rustfmt")
            .arg("--edition")
            .arg("2018")
            .arg(rs_file.to_str().unwrap())
            .output()
        {
            println!(
                "Warning: Unable to auto-format {} using rustfmt: {:?}",
                rs_file.file_name().unwrap().to_str().unwrap(),
                e
            )
        }
    }

    Ok(())
}

// Generate rust bindings for the given ComponentInterface, as a string.

pub fn generate_rust_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    RustWrapper::new(config.clone(), ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render rust bindings"))
}

/// Execute the specifed rust script, with environment based on the generated
/// artifacts in the given output directory.
pub fn run_script(_out_dir: &Path, script_file: &Path) -> Result<()> {
    let mut cmd = Command::new("rustc");

    cmd.arg(script_file);
    let status = cmd
        .spawn()
        .context("Failed to spawn `rustc` when running script")?
        .wait()
        .context("Failed to wait for `rustc` when running script")?;
    if !status.success() {
        bail!("running `rustc` failed")
    }
    Ok(())
}
