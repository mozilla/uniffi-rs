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
    macro_metadata, overridden_config_value, BindgenCrateConfigSupplier, BindingGenerator,
    Component, ComponentInterface, GenerationSettings, Result,
};
use anyhow::{bail, Context};
use camino::Utf8Path;
use std::{collections::BTreeMap, fs};
use toml::value::Table as TomlTable;
use uniffi_meta::{
    create_metadata_groups, group_metadata, Metadata, MetadataGroup, NamespaceMetadata,
};

/// Generate foreign bindings
///
/// This replicates the current process used for generating the builtin bindings.
/// External bindings authors should consider using [find_components], which provides a simpler
/// interface and allows for more flexibility in how the external bindings are generated.
///
/// Returns the list of sources used to generate the bindings, in no particular order.
///
/// Deprecated: External crates are encouraged to use the `BindgenLoader` type instead, which lets
/// you control the binding generation process more directly.
pub fn generate_bindings<T: BindingGenerator>(
    library_path: &Utf8Path,
    crate_name: Option<String>,
    binding_generator: &T,
    config_supplier: &dyn BindgenCrateConfigSupplier,
    config_file_override: Option<&Utf8Path>,
    out_dir: &Utf8Path,
    try_format_code: bool,
) -> Result<Vec<Component<T::Config>>> {
    let mut components = find_components(library_path, config_supplier)
        .with_context(|| format!("finding components in '{library_path}'"))?
        .into_iter()
        .map(|Component { ci, config }| {
            let toml_value = overridden_config_value(config, config_file_override)?;
            let config = binding_generator
                .new_config(&toml_value)
                .context("loading toml")?;
            Ok(Component { ci, config })
        })
        .collect::<Result<Vec<_>>>()?;

    let settings = GenerationSettings {
        out_dir: out_dir.to_owned(),
        try_format_code,
        cdylib: calc_cdylib_name(library_path).map(ToOwned::to_owned),
    };
    binding_generator.update_component_configs(&settings, &mut components)?;

    // need to derive ffi after the bindings have had a chance to update any names etc.
    for component in &mut components {
        component
            .ci
            .derive_ffi_funcs()
            .context("Failed to derive FFI functions")?;
    }

    // give every CI a cloned copy of every CI - including itself for simplicity.
    // we end up taking n^2 copies of all ci's, but it works.
    let all_cis = components.iter().map(|c| c.ci.clone()).collect::<Vec<_>>();
    components
        .iter_mut()
        .for_each(|c| c.ci.set_all_component_interfaces(all_cis.clone()));

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

/// Find UniFFI components from a shared library file
///
/// This method inspects the library file and creates [ComponentInterface] instances for each
/// component used to build it.  It parses the UDL files from `uniffi::include_scaffolding!` macro
/// calls.
///
/// `config_supplier` is used to find UDL files on disk and load config data.
pub fn find_cis(
    library_path: &Utf8Path,
    config_supplier: &dyn BindgenCrateConfigSupplier,
) -> Result<Vec<ComponentInterface>> {
    let items = macro_metadata::extract_from_library(library_path)?;
    let mut metadata_groups = create_metadata_groups(&items);
    group_metadata(&mut metadata_groups, items)?;

    for group in metadata_groups.values_mut() {
        let crate_name = group.namespace.crate_name.clone();
        if let Some(udl_group) = load_udl_metadata(group, &crate_name, config_supplier)? {
            let mut udl_items = udl_group.items.into_iter().collect();
            group.items.append(&mut udl_items);
            if group.namespace_docstring.is_none() {
                group.namespace_docstring = udl_group.namespace_docstring;
            }
        };
    }

    let crate_to_namespace_map: BTreeMap<String, NamespaceMetadata> = metadata_groups
        .iter()
        .map(|(k, v)| (k.clone(), v.namespace.clone()))
        .collect();

    metadata_groups
        .into_values()
        .map(|group| {
            let crate_name = &group.namespace.crate_name;
            let mut ci = ComponentInterface::new(crate_name);
            ci.add_metadata(group)?;
            ci.set_crate_to_namespace_map(crate_to_namespace_map.clone());
            Ok(ci)
        })
        .collect()
}

/// Find UniFFI components from a shared library file
///
/// This method inspects the library file and creates [ComponentInterface] instances for each
/// component used to build it.  It parses the UDL files from `uniffi::include_scaffolding!` macro
/// calls.
///
/// `config_supplier` is used to find UDL files on disk and load config data.
pub fn find_components(
    library_path: &Utf8Path,
    config_supplier: &dyn BindgenCrateConfigSupplier,
) -> Result<Vec<Component<TomlTable>>> {
    find_cis(library_path, config_supplier)?
        .into_iter()
        .map(|ci| {
            let config = config_supplier
                .get_toml(ci.crate_name())?
                .unwrap_or_default();
            Ok(Component { ci, config })
        })
        .collect()
}

fn load_udl_metadata(
    group: &MetadataGroup,
    crate_name: &str,
    config_supplier: &dyn BindgenCrateConfigSupplier,
) -> Result<Option<MetadataGroup>> {
    let udl_items = group
        .items
        .iter()
        .filter_map(|i| match i {
            Metadata::UdlFile(meta) => Some(meta),
            _ => None,
        })
        .collect::<Vec<_>>();
    // We only support 1 UDL file per crate, for no good reason!
    match udl_items.len() {
        0 => Ok(None),
        1 => {
            if udl_items[0].module_path != crate_name {
                bail!(
                    "UDL is for crate '{}' but this crate name is '{}'",
                    udl_items[0].module_path,
                    crate_name
                );
            }
            let udl = config_supplier.get_udl(crate_name, &udl_items[0].file_stub)?;
            let udl_group = uniffi_udl::parse_udl(&udl, crate_name)?;
            Ok(Some(udl_group))
        }
        n => bail!("{n} UDL files found for {crate_name}"),
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
