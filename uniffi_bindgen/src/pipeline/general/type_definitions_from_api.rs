/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add TypeDefinitions for types used in the API as arguments, return types, throws types, etc.
//!
//! Most bindings will need to define FFI converters for these.

use indexmap::IndexSet;

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    // Map types to type definitions to add
    let mut all_types = IndexSet::<Type>::default();
    namespace.visit(|ty: &Type| {
        collect_all_types(&mut all_types, ty);
    });
    for ty in all_types {
        match &ty {
            Type::UInt8
            | Type::Int8
            | Type::UInt16
            | Type::Int16
            | Type::UInt32
            | Type::Int32
            | Type::UInt64
            | Type::Int64
            | Type::Float32
            | Type::Float64
            | Type::Boolean
            | Type::String
            | Type::Bytes
            | Type::Timestamp
            | Type::Duration => {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Simple(TypeNode {
                        ty,
                        ..TypeNode::default()
                    }));
            }
            Type::Optional { inner_type } => {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Optional(OptionalType {
                        inner: TypeNode {
                            ty: (**inner_type).clone(),
                            ..TypeNode::default()
                        },
                        self_type: TypeNode {
                            ty,
                            ..TypeNode::default()
                        },
                    }));
            }
            Type::Sequence { inner_type } => {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Sequence(SequenceType {
                        inner: TypeNode {
                            ty: (**inner_type).clone(),
                            ..TypeNode::default()
                        },
                        self_type: TypeNode {
                            ty,
                            ..TypeNode::default()
                        },
                    }));
            }
            Type::Map {
                key_type,
                value_type,
            } => {
                namespace
                    .type_definitions
                    .push(TypeDefinition::Map(MapType {
                        key: TypeNode {
                            ty: (**key_type).clone(),
                            ..TypeNode::default()
                        },
                        value: TypeNode {
                            ty: (**value_type).clone(),
                            ..TypeNode::default()
                        },
                        self_type: TypeNode {
                            ty,
                            ..TypeNode::default()
                        },
                    }));
            }
            Type::Record {
                namespace: namespace_name,
                name,
                ..
            }
            | Type::Enum {
                namespace: namespace_name,
                name,
                ..
            }
            | Type::Interface {
                namespace: namespace_name,
                name,
                ..
            }
            | Type::Custom {
                namespace: namespace_name,
                name,
                ..
            } if *namespace_name != namespace.name => {
                namespace
                    .type_definitions
                    .push(TypeDefinition::External(ExternalType {
                        namespace: namespace_name.clone(),
                        name: name.clone(),
                        self_type: TypeNode {
                            ty: ty.clone(),
                            ..TypeNode::default()
                        },
                    }))
            }
            _ => (),
        }
    }
    Ok(())
}

fn collect_all_types(all_types: &mut IndexSet<Type>, ty: &Type) {
    all_types.insert(ty.clone());
    match ty {
        Type::Optional { inner_type } => collect_all_types(all_types, inner_type.as_ref()),
        Type::Sequence { inner_type } => collect_all_types(all_types, inner_type.as_ref()),
        Type::Map {
            key_type,
            value_type,
        } => {
            collect_all_types(all_types, key_type.as_ref());
            collect_all_types(all_types, value_type.as_ref());
        }
        Type::Custom { builtin, .. } => collect_all_types(all_types, builtin.as_ref()),
        _ => (),
    }
}
