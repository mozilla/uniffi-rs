/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Initial IR, this is essentially the Metadata from uniffi_meta without changes.

use std::fs;

mod from_uniffi_meta;
mod nodes;
pub use nodes::*;

use anyhow::Result;
use camino::Utf8Path;

use crate::{crate_name_from_cargo_toml, interface, macro_metadata, BindgenCrateConfigSupplier};
use from_uniffi_meta::UniffiMetaConverter;

impl Root {
    pub fn from_library(
        config_supplier: impl BindgenCrateConfigSupplier,
        path: &Utf8Path,
        crate_name: Option<String>,
    ) -> Result<Root> {
        let mut metadata_converter = UniffiMetaConverter::default();
        let mut all_metadata = macro_metadata::extract_from_library(path)?;
        if let Some(crate_name) = crate_name {
            all_metadata.retain(|meta| meta.module_path().split("::").next() == Some(&crate_name));
        }

        let mut udl_to_load = vec![];

        for meta in macro_metadata::extract_from_library(path)? {
            match meta {
                uniffi_meta::Metadata::UdlFile(udl) => {
                    udl_to_load.push((
                        config_supplier.get_udl(&udl.module_path, &udl.file_stub)?,
                        udl.module_path,
                    ));
                }
                meta => metadata_converter.add_metadata_item(meta)?,
            }
        }

        for (udl, module_path) in udl_to_load {
            Self::add_metadata_from_udl(&mut metadata_converter, &udl, &module_path, true)?;
        }
        let mut root = metadata_converter.try_into_initial_ir()?;
        root.cdylib = Some(path.to_string());
        Ok(root)
    }

    pub fn from_udl(path: &Utf8Path, crate_name: Option<String>) -> Result<Root> {
        let mut metadata_converter = UniffiMetaConverter::default();
        let crate_name = match crate_name {
            Some(c) => c,
            None => crate_name_from_cargo_toml(path)?,
        };
        Self::add_metadata_from_udl(
            &mut metadata_converter,
            &fs::read_to_string(path)?,
            &crate_name,
            false,
        )?;
        metadata_converter.try_into_initial_ir()
    }

    fn add_metadata_from_udl(
        metadata_converter: &mut UniffiMetaConverter,
        udl: &str,
        crate_name: &str,
        library_mode: bool,
    ) -> Result<()> {
        let metadata_group = uniffi_udl::parse_udl(udl, crate_name)?;
        // parse_udl returns a metadata group, which is nice for the CI, but we actually want to
        // start with a raw metadata list
        if let Some(docstring) = metadata_group.namespace_docstring {
            metadata_converter
                .add_module_docstring(metadata_group.namespace.name.clone(), docstring);
        }
        metadata_converter
            .add_metadata_item(uniffi_meta::Metadata::Namespace(metadata_group.namespace))?;
        for mut meta in metadata_group.items {
            // some items are both in UDL and library metadata. For many that's fine but
            // uniffi-traits aren't trivial to compare meaning we end up with dupes.
            // We filter out such problematic items here.
            if library_mode && matches!(meta, uniffi_meta::Metadata::UniffiTrait { .. }) {
                continue;
            }
            // Make sure metadata checksums are set
            match &mut meta {
                uniffi_meta::Metadata::Func(func) => {
                    func.checksum = Some(uniffi_meta::checksum(&interface::Function::from(
                        func.clone(),
                    )));
                }
                uniffi_meta::Metadata::Method(meth) => {
                    meth.checksum = Some(uniffi_meta::checksum(&interface::Method::from(
                        meth.clone(),
                    )));
                }
                uniffi_meta::Metadata::Constructor(cons) => {
                    cons.checksum = Some(uniffi_meta::checksum(&interface::Constructor::from(
                        cons.clone(),
                    )));
                }
                // Note: UDL-based callbacks don't have checksum functions, don't set the
                // checksum for those.
                _ => (),
            }
            metadata_converter.add_metadata_item(meta)?;
        }
        Ok(())
    }
}
