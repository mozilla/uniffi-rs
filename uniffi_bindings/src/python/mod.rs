/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::process::Command;

use anyhow::Result;
use fs_err as fs;

mod gen_python;
#[cfg(feature = "bindgen-tests")]
pub mod test;
use uniffi_bindgen::{BindingGenerator, Component, GenerationSettings};

use gen_python::{generate_python_bindings, Config};

pub struct PythonBindingGenerator;

impl BindingGenerator for PythonBindingGenerator {
    type Config = Config;

    fn new_config(&self, root_toml: &toml::Value) -> Result<Self::Config> {
        Ok(
            match root_toml.get("bindings").and_then(|b| b.get("python")) {
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
            let py_file = settings.out_dir.join(format!("{}.py", ci.namespace()));
            fs::write(&py_file, generate_python_bindings(config, &mut ci.clone())?)?;

            if settings.try_format_code {
                if let Err(e) = Command::new("yapf").arg(&py_file).output() {
                    println!(
                        "Warning: Unable to auto-format {} using yapf: {e:?}",
                        py_file.file_name().unwrap(),
                    )
                }
            }
        }

        Ok(())
    }
}
