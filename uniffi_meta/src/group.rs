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

        let crate_name = calc_crate_name(item.module_path()).to_owned();
        let item = convert_external_metadata_item(item);
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

#[derive(Debug)]
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

pub fn convert_external_metadata_item(item: Metadata) -> Metadata {
    let owner_module_path = calc_crate_name(item.module_path()).to_owned();

    let convert_external_type = |ty: Type| do_external_type_conversion(ty, &owner_module_path);

    let converter = TypeConverter::new(convert_external_type);
    converter.convert_item(item)
}

pub fn convert_external_type(ty: Type, owner_module_path: &str) -> Type {
    let convert_external_type = |ty: Type| do_external_type_conversion(ty, owner_module_path);

    let converter = TypeConverter::new(convert_external_type);
    converter.convert_type(ty)
}

// The actual logic for external type conversions.
fn do_external_type_conversion(ty: Type, owner_module_path: &str) -> Type {
    let is_external = |module_path: &str| calc_crate_name(module_path) != owner_module_path;

    match ty {
        // Convert `ty` if it's external
        Type::Enum { module_path, name }
        | Type::Record { module_path, name }
        | Type::Custom {
            module_path, name, ..
        } if is_external(&module_path) => Type::External {
            module_path,
            name,
            kind: ExternalKind::DataClass,
        },
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
                module_path,
                name,
                kind,
            }
        }
        Type::CallbackInterface { module_path, name } if is_external(&module_path) => {
            panic!("External callback interfaces not supported ({name})")
        }

        // Existing External types need namespace fixed.
        Type::External {
            module_path,
            name,
            kind,
            ..
        } => Type::External {
            module_path,
            name,
            kind,
        },
        // Otherwise, just return the type unchanged
        _ => ty,
    }
}

// Can walk a metadata item and replace all types via a user defined function.
struct TypeConverter<F> {
    converter: F,
}

impl<F> TypeConverter<F>
where
    F: Fn(Type) -> Type,
{
    fn new(converter: F) -> Self {
        Self { converter }
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
        // first, we recurse
        let ty = match ty {
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
            _ => ty,
        };
        // then convert
        (self.converter)(ty)
    }
}

fn calc_crate_name(module_path: &str) -> &str {
    module_path.split("::").next().unwrap()
}
