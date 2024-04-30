/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{BindingGenerator, ComponentInterface};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::process::Command;

mod gen_kotlin;
use gen_kotlin::{generate_bindings, Config};
mod test;
pub use test::{run_script, run_test};

pub struct KotlinBindingGenerator;
impl BindingGenerator for KotlinBindingGenerator {
    type Config = Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("kotlin")) {
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
        let mut kt_file = full_bindings_path(config, out_dir);
        fs::create_dir_all(&kt_file)?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        fs::write(&kt_file, generate_bindings(config, ci)?)?;
        if try_format_code {
            if let Err(e) = Command::new("ktlint").arg("-F").arg(&kt_file).output() {
                println!(
                    "Warning: Unable to auto-format {} using ktlint: {e:?}",
                    kt_file.file_name().unwrap(),
                );
            }
        }
        Ok(())
    }

    fn check_library_path(&self, library_path: &Utf8Path, cdylib_name: Option<&str>) -> Result<()> {
        if cdylib_name.is_none() {
            anyhow::bail!(
                "Generate bindings for Kotlin requires a cdylib, but {library_path} was given"
            );
        }
        Ok(())
    }
}

fn full_bindings_path(config: &Config, out_dir: &Utf8Path) -> Utf8PathBuf {
    let package_path: Utf8PathBuf = config.package_name().split('.').collect();
    Utf8PathBuf::from(out_dir).join(package_path)
}
