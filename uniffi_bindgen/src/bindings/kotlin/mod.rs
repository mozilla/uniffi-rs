/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    bindings::GenerateOptions,
    interface::{apply_exclusions, rename},
    BindgenLoader, Component, ComponentInterface, Result,
};
use anyhow::bail;
use camino::{Utf8Path, Utf8PathBuf};
use fs_err as fs;
use std::collections::HashMap;
use std::process::Command;

mod gen_kotlin;
use gen_kotlin::{generate_bindings, Config};
#[cfg(feature = "bindgen-tests")]
pub mod test;

/// Generate Kotlin bindings
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
    apply_renames(&mut components);

    // Check for primary constructors after `apply_renames` is called, so that we honor exclusions.
    for c in components.iter() {
        for o in c.ci.object_definitions() {
            for cons in o.constructors() {
                if cons.is_async() && cons.is_primary_constructor() {
                    bail!(
                        "Async primary constructors not supported but {} has one",
                        o.name()
                    );
                }
            }
        }
    }
    for c in components.iter_mut() {
        // Call derive_ffi_functions after `apply_renames`
        c.ci.derive_ffi_funcs()?;
    }

    for Component { ci, config, .. } in components {
        if let Some(crate_filter) = &options.crate_filter {
            if ci.crate_name() != crate_filter {
                continue;
            }
        }

        let mut kt_file = full_bindings_path(&config, &options.out_dir);
        fs::create_dir_all(&kt_file)?;
        kt_file.push(format!("{}.kt", ci.namespace()));
        fs::write(&kt_file, generate_bindings(&config, &ci)?)?;
        if options.format {
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

fn full_bindings_path(config: &Config, out_dir: &Utf8Path) -> Utf8PathBuf {
    let package_path: Utf8PathBuf = config.package_name().split('.').collect();
    Utf8PathBuf::from(out_dir).join(package_path)
}

fn parse_config(
    ci: &ComponentInterface,
    root_toml: toml::Value,
    cdylib: Option<String>,
) -> Result<Config> {
    let mut config: Config = match root_toml.get("bindings").and_then(|b| b.get("kotlin")) {
        Some(v) => v.clone().try_into()?,
        None => Default::default(),
    };
    config
        .package_name
        .get_or_insert_with(|| format!("uniffi.{}", ci.namespace()));
    config.cdylib_name.get_or_insert_with(|| {
        cdylib
            .clone()
            .unwrap_or_else(|| format!("uniffi_{}", ci.namespace()))
    });

    Ok(config)
}

// A helper for renaming items.
fn apply_renames(components: &mut Vec<Component<Config>>) {
    // Remove excluded items, this happens before renaming
    for c in components.iter_mut() {
        apply_exclusions(&mut c.ci, &c.config.exclude);
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
            if ext_crate != c.ci.crate_name() && !c.config.external_packages.contains_key(ext_crate)
            {
                c.config
                    .external_packages
                    .insert(ext_crate.to_string(), ext_package.clone());
            }
        }
    }
}
