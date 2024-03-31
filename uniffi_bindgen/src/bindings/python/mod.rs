/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

use anyhow::Result;
use camino::Utf8Path;
use fs_err as fs;

mod gen_python;
mod test;
use super::super::interface::ComponentInterface;
use gen_python::{generate_python_bindings, Config};
pub use test::{run_script, run_test};

pub struct PythonBindingGenerator;

impl crate::BindingGenerator for PythonBindingGenerator {
    type Config = Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("python")) {
                Some(v) => v.clone().try_into()?,
                None => Default::default(),
            },
        )
    }

    fn write_bindings(
        &self,
        ci: &ComponentInterface,
        config: &Config,
        out_dir: &Utf8Path,
        try_format_code: bool,
    ) -> Result<()> {
        let py_file = out_dir.join(format!("{}.py", ci.namespace()));
        fs::write(&py_file, generate_python_bindings(config, ci)?)?;

        if try_format_code {
            if let Err(e) = Command::new("yapf").arg(&py_file).output() {
                println!(
                    "Warning: Unable to auto-format {} using yapf: {e:?}",
                    py_file.file_name().unwrap(),
                )
            }
        }

        Ok(())
    }

    fn check_library_path(&self, library_path: &Utf8Path, cdylib_name: Option<&str>) -> Result<()> {
        if cdylib_name.is_none() {
            anyhow::bail!(
                "Generate bindings for Python requires a cdylib, but {library_path} was given"
            );
        }
        Ok(())
    }
}
