/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

use crate::{bindings::GenerateOptions, BindgenLoader, Component, ComponentInterface};
use anyhow::{bail, Context, Result};
use fs_err as fs;

mod gen_ruby;
#[cfg(feature = "bindgen-tests")]
pub mod test;
use gen_ruby::{Config, RubyWrapper};

pub fn generate(loader: &BindgenLoader, options: GenerateOptions) -> Result<()> {
    let metadata = loader.load_metadata(&options.source)?;
    if let Some(crate_filter) = &options.crate_filter {
        if !metadata.contains_key(crate_filter) {
            bail!("No UniFFI metadata found for crate {crate_filter}");
        }
    }
    let cis = loader.load_cis(metadata)?;
    let cdylib = loader.library_name(&options.source).map(|l| l.to_string());
    let mut components =
        loader.load_components(cis, |ci, toml| parse_config(ci, toml, cdylib.clone()))?;
    for c in components.iter_mut() {
        c.ci.derive_ffi_funcs()?;
    }
    for Component { ci, config, .. } in components {
        if let Some(crate_filter) = &options.crate_filter {
            if ci.crate_name() != crate_filter {
                continue;
            }
        }
        let rb_file = options.out_dir.join(format!("{}.rb", ci.namespace()));
        fs::write(&rb_file, generate_ruby_bindings(&config, &ci)?)?;

        if options.format {
            if let Err(e) = Command::new("rubocop").arg("-A").arg(&rb_file).output() {
                println!(
                    "Warning: Unable to auto-format {} using rubocop: {e:?}",
                    rb_file.file_name().unwrap(),
                )
            }
        }
    }
    Ok(())
}

// Generate ruby bindings for the given ComponentInterface, as a string.
pub fn generate_ruby_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use askama::Template;
    RubyWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render ruby bindings")
}

fn parse_config(
    ci: &ComponentInterface,
    root_toml: toml::Value,
    cdylib: Option<String>,
) -> Result<Config> {
    let mut config: Config = match root_toml.get("bindings").and_then(|b| b.get("ruby")) {
        Some(v) => v.clone().try_into()?,
        None => Default::default(),
    };
    config.cdylib_name.get_or_insert_with(|| {
        cdylib
            .clone()
            .unwrap_or_else(|| format!("uniffi_{}", ci.namespace()))
    });
    Ok(config)
}
