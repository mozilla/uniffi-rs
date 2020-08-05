/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    env,
    ffi::OsString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};

pub mod gen_python;
pub use gen_python::{Config, PythonWrapper};

use super::super::interface::ComponentInterface;

// Generate python bindings for the given ComponentInterface, in the given output directory.

pub fn write_bindings(ci: &ComponentInterface, out_dir: &Path) -> Result<()> {
    let mut py_file = PathBuf::from(out_dir);
    py_file.push(format!("{}.py", ci.namespace()));
    let mut f = File::create(&py_file).context("Failed to create .py file for bindings")?;
    write!(f, "{}", generate_python_bindings(&ci)?)?;
    Ok(())
}

// Generate python bindings for the given ComponentInterface, as a string.

pub fn generate_python_bindings(ci: &ComponentInterface) -> Result<String> {
    let config = Config::from(&ci);
    use askama::Template;
    PythonWrapper::new(config, &ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render python bindings"))
}

/// Execute the specifed python script, with environment based on the generated
/// artifacts in the given output directory.
pub fn run_script(out_dir: &Path, script_file: &Path) -> Result<()> {
    let mut pythonpath = env::var_os("PYTHONPATH").unwrap_or_else(|| OsString::from(""));
    // This lets python find the compiled library for the rust component.
    pythonpath.push(":");
    pythonpath.push(out_dir);
    let mut cmd = Command::new("python3");
    cmd.env("PYTHONPATH", pythonpath);
    cmd.arg(script_file);
    let status = cmd
        .spawn()
        .context("Failed to spawn `python` when running script")?
        .wait()
        .context("Failed to wait for `python` when running script")?;
    if !status.success() {
        bail!("running `python` failed")
    }
    Ok(())
}
