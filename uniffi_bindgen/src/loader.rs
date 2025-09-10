/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    collections::{BTreeMap, HashMap},
    fs,
};

use anyhow::bail;
use camino::Utf8Path;
use uniffi_meta::{
    create_metadata_groups, group_metadata, Metadata, MetadataGroup, MetadataGroupMap,
    NamespaceMetadata,
};

use crate::{
    crate_name_from_cargo_toml, macro_metadata, BindgenCrateConfigSupplier, Component,
    ComponentInterface, Result,
};

/// Load metadata, component interfaces, configuration, etc. for binding generators.
///
/// Bindings generators use this to load all of the inputs they need to render their code.
pub struct BindgenLoader<'a> {
    config_supplier: &'a dyn BindgenCrateConfigSupplier,
}

impl<'config> BindgenLoader<'config> {
    pub fn new(config_supplier: &'config dyn BindgenCrateConfigSupplier) -> Self {
        Self { config_supplier }
    }

    /// Load UniFFI metadata
    ///
    /// The metadata describes the exported interface.
    ///
    /// `source_path` can be:
    ///   - A library file (.so, .dylib, .a, etc).  Metadata will be loaded from the symbol table.
    ///   - A UDL file.  The UDL will be parsed and converted into metadata.
    pub fn load_metadata(&self, source_path: &Utf8Path) -> Result<MetadataGroupMap> {
        self.load_metadata_specialized(source_path, |_, _| Ok(None))
    }

    /// Load UniFFI metadata with a specialized metadata parser
    ///
    /// When loading metadata from a library, the passed-in specialized parser will be used to
    /// parse the data.  The library path and library contents will be passed.  If the parser
    /// returns `Ok(Some(metadata))` then this metadata will be used. If it returns `Ok(None)` than
    /// the default parsing will be used.
    ///
    /// Use this function when you want to support specialized library formats.
    pub fn load_metadata_specialized<P>(
        &self,
        source_path: &Utf8Path,
        specialized_parser: P,
    ) -> Result<MetadataGroupMap>
    where
        P: FnOnce(&Utf8Path, &[u8]) -> Result<Option<Vec<Metadata>>>,
    {
        match source_path.extension() {
            Some(ext) if ext.to_lowercase() == "udl" => {
                let crate_name = crate_name_from_cargo_toml(source_path)?;
                let group = uniffi_udl::parse_udl(&fs::read_to_string(source_path)?, &crate_name)?;
                Ok(HashMap::from([(crate_name, group)]))
            }
            _ => {
                let data = fs::read(source_path)?;
                let items = match specialized_parser(source_path, &data)? {
                    Some(items) => items,
                    None => macro_metadata::extract_from_bytes(&data)?,
                };
                let mut metadata_groups = create_metadata_groups(&items);
                group_metadata(&mut metadata_groups, items)?;

                for group in metadata_groups.values_mut() {
                    let crate_name = group.namespace.crate_name.clone();
                    if let Some(udl_group) = self.load_udl_metadata(group, &crate_name)? {
                        let mut udl_items = udl_group.items.into_iter().collect();
                        group.items.append(&mut udl_items);
                        if group.namespace_docstring.is_none() {
                            group.namespace_docstring = udl_group.namespace_docstring;
                        }
                    };
                }
                Ok(metadata_groups)
            }
        }
    }

    /// Load a [ComponentInterface] list
    ///
    /// This converts the metadata into `ComponentInterface` instances, which contains additional
    /// derived information about the interface, like FFI functions signatures.
    pub fn load_cis(&self, metadata: MetadataGroupMap) -> Result<Vec<ComponentInterface>> {
        let crate_to_namespace_map: BTreeMap<String, NamespaceMetadata> = metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.namespace.clone()))
            .collect();

        let mut ci_list = metadata
            .into_values()
            .map(|group| {
                let crate_name = &group.namespace.crate_name;
                let mut ci = ComponentInterface::new(crate_name);
                ci.add_metadata(group)?;
                ci.set_crate_to_namespace_map(crate_to_namespace_map.clone());
                Ok(ci)
            })
            .collect::<Result<Vec<ComponentInterface>>>()?;

        // give every CI a cloned copy of every CI - including itself for simplicity.
        // we end up taking n^2 copies of all ci's, but it works.
        let ci_list2 = ci_list.clone();
        ci_list
            .iter_mut()
            .for_each(|ci| ci.set_all_component_interfaces(ci_list2.clone()));
        Ok(ci_list)
    }

    fn load_udl_metadata(
        &self,
        group: &MetadataGroup,
        crate_name: &str,
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
                let udl = self
                    .config_supplier
                    .get_udl(crate_name, &udl_items[0].file_stub)?;
                let udl_group = uniffi_udl::parse_udl(&udl, crate_name)?;
                Ok(Some(udl_group))
            }
            n => bail!("{n} UDL files found for {crate_name}"),
        }
    }

    /// Load a [Component] list
    ///
    /// This groups [ComponentInterface] values with configuration data from `uniffi.toml` files.
    /// Pass in a `parse_config` function which parses raw TOML into your language-specific config structure.
    ///
    /// Note: the TOML data will contain the entire config tree.
    /// You probably want to do something like
    /// `toml.get("bindings").and_then(|b| b.get("my-language-key"))`
    /// to extract the data for your bindings.
    pub fn load_components<P, Config>(
        &self,
        cis: Vec<ComponentInterface>,
        mut parse_config: P,
    ) -> Result<Vec<Component<Config>>>
    where
        P: FnMut(&ComponentInterface, toml::Value) -> Result<Config>,
        Config: Default,
    {
        cis.into_iter()
            .map(|ci| {
                let toml = self
                    .config_supplier
                    .get_toml(ci.crate_name())?
                    .unwrap_or_default();
                let config = parse_config(&ci, toml.into())?;
                Ok(Component { ci, config })
            })
            .collect()
    }

    /// Get the basename for a source file
    ///
    /// This will remove any file extension.
    /// For libraries it will remove the leading `lib`.
    pub fn source_basename<'a>(&self, source_path: &'a Utf8Path) -> &'a str {
        let mut basename = match source_path.file_stem() {
            Some(stem) => stem,
            None => source_path.as_str(),
        };
        if !self.is_udl(source_path) {
            basename = match basename.strip_prefix("lib") {
                Some(name) => name,
                None => basename,
            }
        };
        basename
    }

    fn is_udl(&self, source_path: &Utf8Path) -> bool {
        matches!(
            source_path.extension(),
            Some(ext) if ext.to_lowercase() == "udl"
        )
    }
}
