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
use anyhow::bail;
use camino::Utf8Path;
use std::fs;
use uniffi_meta::{
    create_metadata_groups, group_metadata,
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
    package_name: Option<String>,
    crate_root: Option<&Utf8Path>,
) -> Result<Vec<Component<T::Config>>> {
    if !package_name.is_some() || !crate_root.is_some() {
        panic!("This hacked version of library_mode needs a package_name and crate_root");
    }
    let ci = find_components(library_path)?;
    let config = binding_generator
        .new_config(&load_initial_config(crate_root.unwrap(), config_file_override)?)?;
    let mut components = vec![Component {
        ci,
        config,
        package_name,
    }];

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
    library_path: &Utf8Path,
) -> Result<ComponentInterface> {
    let items = macro_metadata::extract_from_library(library_path)?;
    let mut metadata_groups = create_metadata_groups(&items);
    group_metadata(&mut metadata_groups, items)?;

    // bad code but there should only be one
    for group in metadata_groups.into_values() {
        let mut ci = ComponentInterface::new(&group.namespace.crate_name);
        ci.add_metadata(group)?;
        return Ok(ci);
    }
    Err(std::fmt::Error.into())
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
