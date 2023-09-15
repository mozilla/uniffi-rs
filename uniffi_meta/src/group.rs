/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};

use crate::*;
use anyhow::{bail, Result};

/// Group metadata by namespace
pub fn group_metadata(items: Vec<Metadata>) -> Result<Vec<MetadataGroup>> {
    // Map crate names to MetadataGroup instances
    let mut group_map = items
        .iter()
        .filter_map(|i| match i {
            Metadata::Namespace(namespace) => {
                let group = MetadataGroup {
                    namespace: namespace.clone(),
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
                    items: BTreeSet::new(),
                };
                Some((udl.module_path.clone(), group))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    for item in items {
        let (item_desc, module_path) = match &item {
            Metadata::Namespace(_) => continue,
            Metadata::UdlFile(meta) => {
                (format!("udl_file `{}`", meta.namespace), &meta.module_path)
            }
            Metadata::Func(meta) => (format!("function `{}`", meta.name), &meta.module_path),
            Metadata::Constructor(meta) => (
                format!("constructor `{}.{}`", meta.self_name, meta.name),
                &meta.module_path,
            ),
            Metadata::Method(meta) => (
                format!("method `{}.{}`", meta.self_name, meta.name),
                &meta.module_path,
            ),
            Metadata::Record(meta) => (format!("record `{}`", meta.name), &meta.module_path),
            Metadata::Enum(meta) => (format!("enum `{}`", meta.name), &meta.module_path),
            Metadata::Object(meta) => (format!("object `{}`", meta.name), &meta.module_path),
            Metadata::CallbackInterface(meta) => (
                format!("callback interface `{}`", meta.name),
                &meta.module_path,
            ),
            Metadata::TraitMethod(meta) => {
                (format!("trait method`{}`", meta.name), &meta.module_path)
            }
            Metadata::Error(meta) => (format!("error `{}`", meta.name()), meta.module_path()),
            Metadata::CustomType(meta) => (format!("custom `{}`", meta.name), &meta.module_path),
        };

        let crate_name = calc_crate_name(module_path);
        let group = match group_map.get_mut(crate_name) {
            Some(ns) => ns,
            None => bail!("Unknown namespace for {item_desc} ({crate_name})"),
        };
        if group.items.contains(&item) {
            bail!("Duplicate metadata item: {item:?}");
        }
        group.add_item(item);
    }
    Ok(group_map.into_values().collect())
}

#[derive(Debug)]
pub struct MetadataGroup {
    pub namespace: NamespaceMetadata,
    pub items: BTreeSet<Metadata>,
}

impl MetadataGroup {
    pub fn add_item(&mut self, item: Metadata) {
        let converter = ExternalTypeConverter {
            crate_name: &self.namespace.crate_name,
        };
        self.items.insert(converter.convert_item(item));
    }
}

/// Convert metadata items by replacing types from external crates with Type::External
struct ExternalTypeConverter<'a> {
    crate_name: &'a str,
}

impl<'a> ExternalTypeConverter<'a> {
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
            Metadata::Error(meta) => Metadata::Error(match meta {
                ErrorMetadata::Enum { enum_, is_flat } => ErrorMetadata::Enum {
                    enum_: self.convert_enum(enum_),
                    is_flat,
                },
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
        match ty {
            // Convert `ty` if it's external
            Type::Enum { module_path, name } | Type::Record { module_path, name }
                if self.is_module_path_external(&module_path) =>
            {
                Type::External {
                    module_path,
                    name,
                    kind: ExternalKind::DataClass,
                }
            }
            Type::Custom {
                module_path, name, ..
            } if self.is_module_path_external(&module_path) => {
                // For now, it's safe to assume that all custom types are data classes.
                // There's no reason to use a custom type with an interface.
                Type::External {
                    module_path,
                    name,
                    kind: ExternalKind::DataClass,
                }
            }
            Type::Object {
                module_path, name, ..
            } if self.is_module_path_external(&module_path) => Type::External {
                module_path,
                name,
                kind: ExternalKind::Interface,
            },
            Type::CallbackInterface { module_path, name }
                if self.is_module_path_external(&module_path) =>
            {
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
                builtin: Box::new(self.convert_type(*builtin)),
            },
            Type::Optional { inner_type } => Type::Optional {
                inner_type: Box::new(self.convert_type(*inner_type)),
            },
            Type::Sequence { inner_type } => Type::Sequence {
                inner_type: Box::new(self.convert_type(*inner_type)),
            },
            Type::Map {
                key_type,
                value_type,
            } => Type::Map {
                key_type: Box::new(self.convert_type(*key_type)),
                value_type: Box::new(self.convert_type(*value_type)),
            },
            // Otherwise, just return the type unchanged
            _ => ty,
        }
    }

    fn is_module_path_external(&self, module_path: &str) -> bool {
        calc_crate_name(module_path) != self.crate_name
    }
}

fn calc_crate_name(module_path: &str) -> &str {
    module_path.split("::").next().unwrap()
}
