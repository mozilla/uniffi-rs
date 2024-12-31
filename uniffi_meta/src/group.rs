/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use crate::*;
use anyhow::{bail, Result};

type MetadataGroupMap = HashMap<String, MetadataGroup>;

// Create empty metadata groups based on the metadata items.
pub fn create_metadata_groups(items: &[Metadata]) -> MetadataGroupMap {
    // Map crate names to MetadataGroup instances
    items
        .iter()
        .filter_map(|i| match i {
            Metadata::Namespace(namespace) => {
                let group = MetadataGroup {
                    namespace: namespace.clone(),
                    namespace_docstring: None,
                    items: BTreeSet::new(),
                };
                Some((namespace.crate_name.clone(), group))
            }
            Metadata::UdlFile(udl) => {
                let namespace = NamespaceMetadata {
                    crate_name: udl.module_path.clone(),
                    name: udl.namespace.clone(),
                };
                let group = MetadataGroup {
                    namespace,
                    namespace_docstring: None,
                    items: BTreeSet::new(),
                };
                Some((udl.module_path.clone(), group))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>()
}

/// Consume the items into the previously created metadata groups.
pub fn group_metadata(group_map: &mut MetadataGroupMap, items: Vec<Metadata>) -> Result<()> {
    for item in items {
        if matches!(&item, Metadata::Namespace(_)) {
            continue;
        }

        let crate_name = calc_crate_name(item.module_path()).to_owned(); // XXX - kill clone?

        let item = fixup_external_type(item, group_map);
        let group = match group_map.get_mut(&crate_name) {
            Some(ns) => ns,
            None => bail!("Unknown namespace for {item:?} ({crate_name})"),
        };
        if group.items.contains(&item) {
            bail!("Duplicate metadata item: {item:?}");
        }
        group.add_item(item);
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct MetadataGroup {
    pub namespace: NamespaceMetadata,
    pub namespace_docstring: Option<String>,
    pub items: BTreeSet<Metadata>,
}

impl MetadataGroup {
    pub fn add_item(&mut self, item: Metadata) {
        self.items.insert(item);
    }
}

pub fn fixup_external_type(item: Metadata, group_map: &MetadataGroupMap) -> Metadata {
    let crate_name = calc_crate_name(item.module_path()).to_owned();
    let converter = ExternalTypeConverter {
        crate_name: &crate_name,
        crate_to_namespace: group_map,
    };
    converter.convert_item(item)
}

/// Convert metadata items by replacing types from external crates with Type::External
struct ExternalTypeConverter<'a> {
    crate_name: &'a str,
    crate_to_namespace: &'a MetadataGroupMap,
}

impl<'a> ExternalTypeConverter<'a> {
    fn crate_to_namespace(&self, crate_name: &str) -> String {
        self.crate_to_namespace
            .get(crate_name)
            .unwrap_or_else(|| panic!("Can't find namespace for module {crate_name}"))
            .namespace
            .name
            .clone()
    }

    fn convert_item(&self, item: Metadata) -> Metadata {
        match item {
            Metadata::Func(meta) => Metadata::Func(FnMetadata {
                inputs: self.convert_params(meta.inputs),
                return_type: self.convert_optional(meta.return_type),
                throws: self.convert_optional(meta.throws),
                ..meta
            }),
            Metadata::Method(meta) => Metadata::Method(MethodMetadata {
                inputs: self.convert_params(meta.inputs),
                return_type: self.convert_optional(meta.return_type),
                throws: self.convert_optional(meta.throws),
                ..meta
            }),
            Metadata::TraitMethod(meta) => Metadata::TraitMethod(TraitMethodMetadata {
                inputs: self.convert_params(meta.inputs),
                return_type: self.convert_optional(meta.return_type),
                throws: self.convert_optional(meta.throws),
                ..meta
            }),
            Metadata::Constructor(meta) => Metadata::Constructor(ConstructorMetadata {
                inputs: self.convert_params(meta.inputs),
                throws: self.convert_optional(meta.throws),
                ..meta
            }),
            Metadata::Record(meta) => Metadata::Record(RecordMetadata {
                fields: self.convert_fields(meta.fields),
                ..meta
            }),
            Metadata::Enum(meta) => Metadata::Enum(self.convert_enum(meta)),
            Metadata::CustomType(meta) => Metadata::CustomType(CustomTypeMetadata {
                builtin: self.convert_type(meta.builtin),
                ..meta
            }),
            _ => item,
        }
    }

    fn convert_params(&self, params: Vec<FnParamMetadata>) -> Vec<FnParamMetadata> {
        params
            .into_iter()
            .map(|param| FnParamMetadata {
                ty: self.convert_type(param.ty),
                ..param
            })
            .collect()
    }

    fn convert_fields(&self, fields: Vec<FieldMetadata>) -> Vec<FieldMetadata> {
        fields
            .into_iter()
            .map(|field| FieldMetadata {
                ty: self.convert_type(field.ty),
                ..field
            })
            .collect()
    }

    fn convert_enum(&self, enum_: EnumMetadata) -> EnumMetadata {
        EnumMetadata {
            variants: enum_
                .variants
                .into_iter()
                .map(|variant| VariantMetadata {
                    fields: self.convert_fields(variant.fields),
                    ..variant
                })
                .collect(),
            ..enum_
        }
    }

    fn convert_optional(&self, ty: Option<Type>) -> Option<Type> {
        ty.map(|ty| self.convert_type(ty))
    }

    fn convert_type(&self, ty: Type) -> Type {
        convert_external_type(ty, self.crate_name, &|mod_path| {
            self.crate_to_namespace(mod_path)
        })
    }
}

// If a type is not owned by "owner_module_path", convert it to an external type. If that conversion
// happens we'll call the closure to find the correct namespace for the external crate.
pub fn convert_external_type<F>(ty: Type, owner_module_path: &str, crate_to_namespace: &F) -> Type
where
    F: Fn(&str) -> String,
{
    let is_external = |module_path: &str| calc_crate_name(module_path) != owner_module_path;
    match ty {
        // Convert `ty` if it's external
        Type::Enum { module_path, name } | Type::Record { module_path, name }
            if is_external(&module_path) =>
        {
            Type::External {
                namespace: crate_to_namespace(&module_path),
                module_path,
                name,
                kind: ExternalKind::DataClass,
            }
        }
        Type::Custom {
            module_path, name, ..
        } if is_external(&module_path) => {
            // For now, it's safe to assume that all custom types are data classes.
            // There's no reason to use a custom type with an interface.
            Type::External {
                namespace: crate_to_namespace(&module_path),
                module_path,
                name,
                kind: ExternalKind::DataClass,
            }
        }
        Type::Object {
            module_path,
            name,
            imp,
        } if is_external(&module_path) => {
            let kind = match imp {
                ObjectImpl::Struct => ExternalKind::Interface,
                ObjectImpl::Trait => ExternalKind::Trait,
                ObjectImpl::CallbackTrait => ExternalKind::Trait,
            };
            Type::External {
                namespace: crate_to_namespace(&module_path),
                module_path,
                name,
                kind,
            }
        }
        Type::CallbackInterface { module_path, name } if is_external(&module_path) => {
            panic!("External callback interfaces not supported ({name})")
        }
        // Convert child types
        Type::Custom {
            module_path,
            name,
            builtin,
            ..
        } => Type::Custom {
            module_path,
            name,
            builtin: Box::new(convert_external_type(
                *builtin,
                owner_module_path,
                crate_to_namespace,
            )),
        },
        Type::Optional { inner_type } => Type::Optional {
            inner_type: Box::new(convert_external_type(
                *inner_type,
                owner_module_path,
                crate_to_namespace,
            )),
        },
        Type::Sequence { inner_type } => Type::Sequence {
            inner_type: Box::new(convert_external_type(
                *inner_type,
                owner_module_path,
                crate_to_namespace,
            )),
        },
        Type::Map {
            key_type,
            value_type,
        } => Type::Map {
            key_type: Box::new(convert_external_type(
                *key_type,
                owner_module_path,
                crate_to_namespace,
            )),
            value_type: Box::new(convert_external_type(
                *value_type,
                owner_module_path,
                crate_to_namespace,
            )),
        },
        // Existing External types need namespace fixed.
        Type::External {
            module_path,
            name,
            kind,
            ..
        } => Type::External {
            namespace: crate_to_namespace(&module_path),
            module_path,
            name,
            kind,
        },
        // Otherwise, just return the type unchanged
        _ => ty,
    }
}

fn calc_crate_name(module_path: &str) -> &str {
    module_path.split("::").next().unwrap()
}
