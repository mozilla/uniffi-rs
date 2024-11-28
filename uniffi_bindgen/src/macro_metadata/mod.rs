/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{
    crate_name_from_cargo_toml, BindgenCrateConfigSupplier,
    {interface::ComponentInterface, library_mode::load_udl_metadata},
};
use anyhow::{bail, Context, Result};
use camino::Utf8Path;
use std::{collections::HashMap, fs};
use uniffi_meta::{
    create_metadata_groups, fixup_external_type, group_metadata, Metadata, MetadataGroup,
};

mod ci;
mod extract;

pub use ci::{add_group_to_ci, add_to_ci};
pub use extract::extract_from_library;

pub fn add_to_ci_from_library(
    iface: &mut ComponentInterface,
    library_path: &Utf8Path,
) -> Result<()> {
    add_to_ci(
        iface,
        extract_from_library(library_path).context("Failed to extract proc-macro metadata")?,
    )
    .context("Failed to add proc-macro metadata to ComponentInterface")
}

pub fn load_metadata_from_library(
    library_path: &Utf8Path,
    crate_name: Option<&str>,
    config_supplier: impl BindgenCrateConfigSupplier,
) -> Result<Vec<MetadataGroup>> {
    let items = extract_from_library(library_path)?;
    let mut metadata_groups = create_metadata_groups(&items);
    group_metadata(&mut metadata_groups, items)?;

    // Collect and process all UDL from all groups at the start - the fixups
    // of external types makes this tricky to do as we finalize the group.
    let mut udl_items: HashMap<String, MetadataGroup> = HashMap::new();

    for group in metadata_groups.values() {
        let crate_name = group.namespace.crate_name.clone();
        if let Some(mut metadata_group) = load_udl_metadata(group, &crate_name, &config_supplier)? {
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

    for group in metadata_groups.values_mut() {
        if let Some(udl_group) = udl_items.remove(&group.namespace.crate_name) {
            group.items.extend(udl_group.items);
        }
    }

    if let Some(crate_name) = crate_name {
        let filtered: Vec<MetadataGroup> = metadata_groups
            .into_values()
            .filter(|group| group.namespace.crate_name == crate_name)
            .collect();
        match filtered.len() {
            0 => bail!("Crate {crate_name} not found in {library_path}"),
            1 => Ok(filtered),
            n => bail!("{n} crates named {crate_name} found in {library_path}"),
        }
    } else {
        Ok(metadata_groups.into_values().collect())
    }
}

pub fn load_metadata_from_udl(
    udl_path: &Utf8Path,
    crate_name: Option<&str>,
) -> Result<Vec<MetadataGroup>> {
    let crate_name = crate_name
        .map(|c| Ok(c.to_string()))
        .unwrap_or_else(|| crate_name_from_cargo_toml(udl_path))?;
    let udl = fs::read_to_string(udl_path)
        .with_context(|| format!("Failed to read UDL from {udl_path}"))?;
    Ok(vec![uniffi_udl::parse_udl(&udl, &crate_name)?])
}
