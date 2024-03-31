/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

use crate::{BindingGenerator, ComponentInterface};
use anyhow::{Context, Result};
use camino::Utf8Path;
use fs_err as fs;

mod gen_ruby;
mod test;
use gen_ruby::{Config, RubyWrapper};
pub use test::run_test;

pub struct RubyBindingGenerator;
impl BindingGenerator for RubyBindingGenerator {
    type Config = Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("ruby")) {
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
        let rb_file = out_dir.join(format!("{}.rb", ci.namespace()));
        fs::write(&rb_file, generate_ruby_bindings(config, ci)?)?;

        if try_format_code {
            if let Err(e) = Command::new("rubocop").arg("-A").arg(&rb_file).output() {
                println!(
                    "Warning: Unable to auto-format {} using rubocop: {e:?}",
                    rb_file.file_name().unwrap(),
                )
            }
        }

        Ok(())
    }

    fn check_library_path(&self, library_path: &Utf8Path, cdylib_name: Option<&str>) -> Result<()> {
        if cdylib_name.is_none() {
            anyhow::bail!(
                "Generate bindings for Ruby requires a cdylib, but {library_path} was given"
            );
        }
        Ok(())
    }
}

// Generate ruby bindings for the given ComponentInterface, as a string.
pub fn generate_ruby_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    RubyWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render ruby bindings")
}
