/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Alternative implementation for the `generate` command, that we plan to eventually replace the current default with.
///
/// Traditionally, users would invoke `uniffi-bindgen generate` to generate bindings for a single crate, passing it the UDL file, config file, etc.
///
/// library_mode is a new way to generate bindings for multiple crates at once.
/// Users pass the path to the build cdylib file and UniFFI figures everything out, leveraging `cargo_metadata`, the metadata UniFFI stores inside exported symbols in the dylib, etc.
///
/// This brings several advantages.:
///   - No more need to specify the dylib in the `uniffi.toml` file(s)
///   - UniFFI can figure out the dependencies based on the dylib exports and generate the sources for
///     all of them at once.
///   - UniFFI can figure out the package/module names for each crate, eliminating the external
///     package maps.
use crate::{
    load_initial_config, macro_metadata, BindingGenerator, Component, ComponentInterface,
    GenerationSettings, Result,
};
use anyhow::{bail, Context};
use camino::Utf8Path;
use cargo_metadata::{MetadataCommand, Package};
use std::{collections::HashMap, fs};
use uniffi_meta::{
    create_metadata_groups, fixup_external_type, group_metadata, Metadata, MetadataGroup,
};

/// Generate foreign bindings
///
/// Returns the list of sources used to generate the bindings, in no particular order.
// XXX - we should consider killing this function and replace it with a function
// which just locates the `Components` and returns them, leaving the filtering
// and actual generation to the callers, which also would allow removing the potentially
// confusing crate_name param.
pub fn generate_bindings<T: BindingGenerator + ?Sized>(
    library_path: &Utf8Path,
    crate_name: Option<String>,
    binding_generator: &T,
    config_file_override: Option<&Utf8Path>,
    out_dir: &Utf8Path,
    try_format_code: bool,
) -> Result<Vec<Component<T::Config>>> {
    let cargo_metadata = MetadataCommand::new()
        .exec()
        .context("error running cargo metadata")?;

    let mut components = find_components(&cargo_metadata, library_path)?
        .into_iter()
        .map(|(ci, package)| {
            let crate_root = package
                .manifest_path
                .parent()
                .context("manifest path has no parent")?;
            let config = binding_generator
                .new_config(&load_initial_config(crate_root, config_file_override)?)?;
            Ok(Component {
                ci,
                config,
                package_name: Some(package.name),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let settings = GenerationSettings {
        out_dir: out_dir.to_owned(),
        try_format_code,
        cdylib: calc_cdylib_name(library_path).map(ToOwned::to_owned),
    };
    binding_generator.update_component_configs(&settings, &mut components)?;

    fs::create_dir_all(out_dir)?;
    if let Some(crate_name) = &crate_name {
        let old_elements = components.drain(..);
        let mut matches: Vec<_> = old_elements
            .filter(|s| s.ci.crate_name() == crate_name)
            .collect();
        match matches.len() {
            0 => bail!("Crate {crate_name} not found in {library_path}"),
            1 => components.push(matches.pop().unwrap()),
            n => bail!("{n} crates named {crate_name} found in {library_path}"),
        }
    }

    binding_generator.write_bindings(&settings, &components)?;

    Ok(components)
}

// If `library_path` is a C dynamic library, return its name
pub fn calc_cdylib_name(library_path: &Utf8Path) -> Option<&str> {
    let cdylib_extensions = [".so", ".dll", ".dylib"];
    let filename = library_path.file_name()?;
    let filename = filename.strip_prefix("lib").unwrap_or(filename);
    for ext in cdylib_extensions {
        if let Some(f) = filename.strip_suffix(ext) {
            return Some(f);
        }
    }
    None
}

fn find_components(
    cargo_metadata: &cargo_metadata::Metadata,
    library_path: &Utf8Path,
) -> Result<Vec<(ComponentInterface, Package)>> {
    let items = macro_metadata::extract_from_library(library_path)?;
    let mut metadata_groups = create_metadata_groups(&items);
    group_metadata(&mut metadata_groups, items)?;

    // Collect and process all UDL from all groups at the start - the fixups
    // of external types makes this tricky to do as we finalize the group.
    let mut udl_items: HashMap<String, MetadataGroup> = HashMap::new();

    for group in metadata_groups.values() {
        let package = find_package_by_crate_name(cargo_metadata, &group.namespace.crate_name)?;
        let crate_root = package
            .manifest_path
            .parent()
            .context("manifest path has no parent")?;
        let crate_name = group.namespace.crate_name.clone();
        if let Some(mut metadata_group) = load_udl_metadata(group, crate_root, &crate_name)? {
            // fixup the items.
            metadata_group.items = metadata_group
                .items
                .into_iter()
                .map(|item| fixup_external_type(item, &metadata_groups))
                // some items are both in UDL and library metadata. For many that's fine but
                // uniffi-traits aren't trivial to compare meaning we end up with dupes.
                // We filter out such problematic items here.
                .filter(|item| !matches!(item, Metadata::UniffiTrait { .. }))
                .collect();
            udl_items.insert(crate_name, metadata_group);
        };
    }

    metadata_groups
        .into_values()
        .map(|group| {
            let package = find_package_by_crate_name(cargo_metadata, &group.namespace.crate_name)?;
            let mut ci = ComponentInterface::new(&group.namespace.crate_name);
            if let Some(metadata) = udl_items.remove(&group.namespace.crate_name) {
                ci.add_metadata(metadata)?;
            };
            ci.add_metadata(group)?;
            Ok((ci, package))
        })
        .collect()
}

fn find_package_by_crate_name(
    metadata: &cargo_metadata::Metadata,
    crate_name: &str,
) -> Result<Package> {
    let matching: Vec<&Package> = metadata
        .packages
        .iter()
        .filter(|p| {
            p.targets
                .iter()
                .any(|t| t.name.replace('-', "_") == crate_name)
        })
        .collect();
    match matching.len() {
        1 => Ok(matching[0].clone()),
        n => bail!("cargo metadata returned {n} packages for crate name {crate_name}"),
    }
}

fn load_udl_metadata(
    group: &MetadataGroup,
    crate_root: &Utf8Path,
    crate_name: &str,
) -> Result<Option<MetadataGroup>> {
    let udl_items = group
        .items
        .iter()
        .filter_map(|i| match i {
            uniffi_meta::Metadata::UdlFile(meta) => Some(meta),
            _ => None,
        })
        .collect::<Vec<_>>();
    match udl_items.len() {
        // No UDL files, load directly from the group
        0 => Ok(None),
        // Found a UDL file, use it to load the CI, then add the MetadataGroup
        1 => {
            if udl_items[0].module_path != crate_name {
                bail!(
                    "UDL is for crate '{}' but this crate name is '{}'",
                    udl_items[0].module_path,
                    crate_name
                );
            }
            let ci_name = &udl_items[0].file_stub;
            let ci_path = crate_root.join("src").join(format!("{ci_name}.udl"));
            if ci_path.exists() {
                let udl = fs::read_to_string(ci_path)?;
                let udl_group = uniffi_udl::parse_udl(&udl, crate_name)?;
                Ok(Some(udl_group))
            } else {
                bail!("{ci_path} not found");
            }
        }
        n => bail!("{n} UDL files found for {crate_root}"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn calc_cdylib_name_is_correct() {
        assert_eq!(
            "uniffi",
            calc_cdylib_name("/path/to/libuniffi.so".into()).unwrap()
        );
        assert_eq!(
            "uniffi",
            calc_cdylib_name("/path/to/libuniffi.dylib".into()).unwrap()
        );
        assert_eq!(
            "uniffi",
            calc_cdylib_name("/path/to/uniffi.dll".into()).unwrap()
        );
    }

    /// Right now we unconditionally strip the `lib` prefix.
    ///
    /// Technically Windows DLLs do not start with a `lib` prefix,
    /// but a library name could start with a `lib` prefix.
    /// On Linux/macOS this would result in a `liblibuniffi.{so,dylib}` file.
    #[test]
    #[ignore] // Currently fails.
    fn calc_cdylib_name_is_correct_on_windows() {
        assert_eq!(
            "libuniffi",
            calc_cdylib_name("/path/to/libuniffi.dll".into()).unwrap()
        );
    }
}
