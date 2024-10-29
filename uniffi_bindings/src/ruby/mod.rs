/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

use crate::{BindingGenerator, Component, ComponentInterface, GenerationSettings};
use anyhow::{Context, Result};
use fs_err as fs;

mod gen_ruby;
#[cfg(feature = "bindgen-tests")]
pub mod test;
use gen_ruby::{Config, RubyWrapper};

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

    fn update_component_configs(
        &self,
        settings: &GenerationSettings,
        components: &mut Vec<Component<Self::Config>>,
    ) -> Result<()> {
        for c in &mut *components {
            c.config.cdylib_name.get_or_insert_with(|| {
                settings
                    .cdylib
                    .clone()
                    .unwrap_or_else(|| format!("uniffi_{}", c.ci.namespace()))
            });
        }
        Ok(())
    }

    fn write_bindings(
        &self,
        settings: &GenerationSettings,
        components: &[Component<Self::Config>],
    ) -> Result<()> {
        for Component { ci, config, .. } in components {
            let rb_file = settings.out_dir.join(format!("{}.rb", ci.namespace()));
            fs::write(&rb_file, generate_ruby_bindings(config, ci)?)?;

            if settings.try_format_code {
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
}

// Generate ruby bindings for the given ComponentInterface, as a string.
pub fn generate_ruby_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    use rinja::Template;
    RubyWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render ruby bindings")
}
