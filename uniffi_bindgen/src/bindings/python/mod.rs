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

pub fn write_bindings(
    config: &Config,
    ci: &ComponentInterface,
    out_dir: &Path,
    try_format_code: bool,
    _is_testing: bool,
) -> Result<()> {
    let mut py_file = PathBuf::from(out_dir);
    py_file.push(format!("{}.py", ci.namespace()));
    let mut f = File::create(&py_file).context("Failed to create .py file for bindings")?;
    write!(f, "{}", generate_python_bindings(config, &ci)?)?;

    if try_format_code {
        if let Err(e) = Command::new("yapf").arg(py_file.to_str().unwrap()).output() {
            println!(
                "Warning: Unable to auto-format {} using yapf: {:?}",
                py_file.file_name().unwrap().to_str().unwrap(),
                e
            )
        }
    }

    Ok(())
}

// Generate python bindings for the given ComponentInterface, as a string.

pub fn generate_python_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    PythonWrapper::new(config.clone(), &ci)
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
