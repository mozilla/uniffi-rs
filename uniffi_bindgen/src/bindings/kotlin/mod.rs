/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{interface::rename, BindingGenerator, Component, GenerationSettings, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::collections::HashMap;
use std::process::Command;

mod gen_kotlin;
use gen_kotlin::{generate_bindings, Config};
#[cfg(feature = "bindgen-tests")]
pub mod test;

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

    fn update_component_configs(
        &self,
        settings: &GenerationSettings,
        components: &mut Vec<Component<Self::Config>>,
    ) -> Result<()> {
        for c in &mut *components {
            c.config
                .package_name
                .get_or_insert_with(|| format!("uniffi.{}", c.ci.namespace()));
            c.config.cdylib_name.get_or_insert_with(|| {
                settings
                    .cdylib
                    .clone()
                    .unwrap_or_else(|| format!("uniffi_{}", c.ci.namespace()))
            });
        }

        // Collect all rename configurations from all components, keyed by module_path
        let mut module_renames = HashMap::new();
        for c in components.iter() {
            if !c.config.rename.is_empty() {
                let module_path = c.ci.crate_name().to_string();
                module_renames.insert(module_path, c.config.rename.clone());
            }
        }

        // Apply rename configurations to all components
        if !module_renames.is_empty() {
            for c in &mut *components {
                rename(&mut c.ci, &module_renames);
            }
        }
        // We need to update package names
        let packages = HashMap::<String, String>::from_iter(
            components
                .iter()
                .map(|c| (c.ci.crate_name().to_string(), c.config.package_name())),
        );
        for c in components {
            for (ext_crate, ext_package) in &packages {
                if ext_crate != c.ci.crate_name()
                    && !c.config.external_packages.contains_key(ext_crate)
                {
                    c.config
                        .external_packages
                        .insert(ext_crate.to_string(), ext_package.clone());
                }
            }
        }
        Ok(())
    }

    fn write_bindings(
        &self,
        settings: &GenerationSettings,
        components: &[Component<Self::Config>],
    ) -> Result<()> {
        for Component { ci, config, .. } in components {
            let mut kt_file = full_bindings_path(config, &settings.out_dir);
            fs::create_dir_all(&kt_file)?;
            kt_file.push(format!("{}.kt", ci.namespace()));
            fs::write(&kt_file, generate_bindings(config, ci)?)?;
            if settings.try_format_code {
                println!(
                    "Code generation complete, formatting with ktlint (use --no-format to disable)"
                );
                if let Err(e) = Command::new("ktlint").arg("-F").arg(&kt_file).output() {
                    println!(
                        "Warning: Unable to auto-format {} using ktlint: {e:?}",
                        kt_file.file_name().unwrap(),
                    );
                }
            }
        }
        Ok(())
    }
}

fn full_bindings_path(config: &Config, out_dir: &Utf8Path) -> Utf8PathBuf {
    let package_path: Utf8PathBuf = config.package_name().split('.').collect();
    Utf8PathBuf::from(out_dir).join(package_path)
}
