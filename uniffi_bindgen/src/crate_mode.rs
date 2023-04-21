/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Alternative implementation for the `generate` command
///
/// Traditionally, users would invoke `uniffi-bindgen generate` to generate bindings for a single
/// crate, passing it the UDL file, config file, etc.
///
/// crate_mode is a new way to generate bindings for multiple crates at once.  Users pass the name
/// of a crate and UniFFI figures everything out, leveraging `cargo_metadata`, the metadata UniFFI
/// stores inside exported symbols in the dylib, etc.
///
/// This brings several advantages.:
///   - No more need to specify the dylib in the config file.
///   - UniFFI can figure out the dependencies based on the dylib exports and generate the sources for
///     all of them at once.
///   - UniFFI can figure out the package/module names for each crate, eliminating the external
///     package maps.
use crate::{bindings, macro_metadata, parse_udl, ComponentInterface, Config, Result};
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Message, MetadataCommand, Package, Target};
use std::{
    collections::{HashMap, HashSet},
    env::consts::{DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX},
    fs,
    process::{Command, Stdio},
};
use uniffi_meta::group_metadata;

/// Generate foreign bindings
///
/// Returns the list of sources used to generate the bindings.  The main crate is first, followed
/// by all other crates in no particular order.
pub fn generate_bindings(
    main_crate_name: &str,
    target_languages: &[String],
    out_dir: &Utf8Path,
    try_format_code: bool,
) -> Result<Vec<Source>> {
    let cargo_metadata = MetadataCommand::new()
        .exec()
        .context("error running cargo metadata")?;
    let main_crate_package = find_package(&cargo_metadata, main_crate_name)?;
    let library_path = find_cdylib_path(main_crate_name, &main_crate_package)?;
    let mut sources = calc_sources(&cargo_metadata, &library_path)?;
    for i in 0..sources.len() {
        // Partition up the sources list because we're eventually going to call
        // `update_from_dependency_configs()` which requires an exclusive reference to one source and
        // shared references to all other sources.
        let (sources_before, rest) = sources.split_at_mut(i);
        let (source, sources_after) = rest.split_first_mut().unwrap();
        let other_sources = sources_before.iter().chain(sources_after.iter());
        // Calculate which configs come from dependent crates
        let dependencies =
            HashSet::<&str>::from_iter(source.package.dependencies.iter().map(|d| d.name.as_str()));
        let config_map: HashMap<&str, &Config> = other_sources
            .filter_map(|s| {
                dependencies
                    .contains(s.package.name.as_str())
                    .then_some((s.crate_name.as_str(), &s.config))
            })
            .collect();
        // We can finally call update_from_dependency_configs
        source.config.update_from_dependency_configs(config_map);
    }
    fs::create_dir_all(out_dir)?;
    // Move the main package to the front
    let main_crate_index = sources
        .iter()
        .position(|s| s.package == main_crate_package)
        .unwrap();
    sources.swap(0, main_crate_index);
    for source in sources.iter() {
        for language in target_languages {
            let language: bindings::TargetLanguage = language.as_str().try_into()?;
            bindings::write_bindings(
                &source.config.bindings,
                &source.ci,
                out_dir,
                language,
                try_format_code,
            )?;
        }
    }

    Ok(sources)
}

// A single source that we generate bindings for
#[derive(Debug)]
pub struct Source {
    pub package: Package,
    pub crate_name: String,
    pub ci: ComponentInterface,
    pub config: Config,
}

fn calc_sources(
    cargo_metadata: &cargo_metadata::Metadata,
    library_path: &Utf8Path,
) -> Result<Vec<Source>> {
    let cdylib_name = library_path
        .file_name()
        .expect("Unexpected library path: {library_path}")
        .strip_prefix(DLL_PREFIX)
        .expect("Unexpected library path: {library_path}")
        .strip_suffix(DLL_SUFFIX)
        .expect("Unexpected library path: {library_path}");
    group_metadata(macro_metadata::extract_from_library(library_path)?)?
        .into_iter()
        .map(|group| {
            let package = find_package_by_crate_name(cargo_metadata, &group.namespace.crate_name)?;
            let crate_root = package
                .manifest_path
                .parent()
                .context("manifest path has no parent")?;
            let mut ci =
                load_component_interface(&group.namespace.crate_name, crate_root, &group.items)?;
            let crate_name = group.namespace.crate_name.clone();
            macro_metadata::add_group_to_ci(&mut ci, group)?;
            let mut config = Config::load_initial(crate_root, None)?;
            config.update_from_cdylib_name(cdylib_name);
            config.update_from_ci(&ci);
            Ok(Source {
                config,
                crate_name,
                ci,
                package,
            })
        })
        .collect()
}

fn find_cdylib_path(main_crate_name: &str, package: &Package) -> Result<Utf8PathBuf> {
    let cdylib_targets: Vec<&Target> = package
        .targets
        .iter()
        .filter(|t| t.crate_types.iter().any(|t| t == "cdylib"))
        .collect();
    let target = match cdylib_targets.len() {
        1 => cdylib_targets[0],
        n => bail!("Found {n} cdylib targets for {}", package.name),
    };

    let messages = build_messages(main_crate_name)?;
    let artifacts = messages.iter().filter_map(|message| match message {
        Message::CompilerArtifact(artifact) => {
            if artifact.target == *target {
                Some(artifact.clone())
            } else {
                None
            }
        }
        _ => None,
    });
    let cdylib_files: Vec<Utf8PathBuf> = artifacts
        .into_iter()
        .flat_map(|artifact| {
            artifact
                .filenames
                .into_iter()
                .filter(|nm| matches!(nm.extension(), Some(DLL_EXTENSION)))
                .collect::<Vec<Utf8PathBuf>>()
        })
        .collect();

    match cdylib_files.len() {
        1 => Ok(cdylib_files[0].to_owned()),
        n => bail!("Found {n} cdylib files for {}", package.name),
    }
}

fn build_messages(main_crate_name: &str) -> Result<Vec<Message>> {
    let mut child = Command::new(env!("CARGO"))
        .args(["build", "-p", main_crate_name])
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .spawn()
        .context("Error running cargo build")?;
    let output = std::io::BufReader::new(child.stdout.take().unwrap());
    Message::parse_stream(output)
        .map(|result| result.map_err(anyhow::Error::from))
        .collect()
}

fn find_package(metadata: &cargo_metadata::Metadata, name: &str) -> Result<Package> {
    let matching: Vec<&Package> = metadata
        .packages
        .iter()
        .filter(|p| p.name == name)
        .collect();
    match matching.len() {
        1 => Ok(matching[0].clone()),
        n => bail!("cargo metadata returned {n} packages named {name}"),
    }
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
                .any(|t| t.name == crate_name && t.crate_types.iter().any(|ct| ct == "lib"))
        })
        .collect();
    match matching.len() {
        1 => Ok(matching[0].clone()),
        n => bail!("cargo metadata returned {n} packages for crate name {crate_name}"),
    }
}

fn load_component_interface(
    crate_name: &str,
    crate_root: &Utf8Path,
    metadata: &[uniffi_meta::Metadata],
) -> Result<ComponentInterface> {
    let udl_items = metadata
        .iter()
        .filter_map(|i| match i {
            uniffi_meta::Metadata::UdlFile(meta) => Some(meta),
            _ => None,
        })
        .collect::<Vec<_>>();
    let ci_name = match udl_items.len() {
        0 => bail!("No UDL files found for {crate_name}"),
        1 => &udl_items[0].name,
        n => bail!("{n} UDL files found for {crate_name}"),
    };
    let ci_path = crate_root.join("src").join(format!("{ci_name}.udl"));
    if ci_path.exists() {
        parse_udl(&ci_path)
    } else {
        bail!("{ci_path} not found");
    }
}
