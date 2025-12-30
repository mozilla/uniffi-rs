/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add TypeDefinitions for types used in the API as arguments, return types, throws types, etc.
//!
//! Most bindings will need to define FFI converters for these.

use indexmap::IndexSet;

use super::*;

pub fn type_definitions(
    namespace: &initial::Namespace,
    context: &Context,
) -> Result<Vec<TypeDefinition>> {
    // Map types to type definitions to add
    let mut all_types = IndexSet::<Type>::default();
    // String is used in the builtin functions, so force it to be present.
    all_types.insert(Type::String);
    namespace.visit(|ty: &Type| {
        all_types.insert(ty.clone());
    });
    let mut type_definitions = vec![];
    for ty in all_types {
        let self_type = ty.clone().map_node(context)?;
        match ty {
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
                type_definitions.push(TypeDefinition::Simple(ty.map_node(context)?));
            }
            Type::Optional { inner_type } => {
                type_definitions.push(TypeDefinition::Optional(OptionalType {
                    inner: (*inner_type).map_node(context)?,
                    self_type,
                }));
            }
            Type::Sequence { inner_type } => {
                type_definitions.push(TypeDefinition::Sequence(SequenceType {
                    inner: (*inner_type).map_node(context)?,
                    self_type,
                }));
            }
            Type::Map {
                key_type,
                value_type,
            } => {
                type_definitions.push(TypeDefinition::Map(MapType {
                    key: (*key_type).map_node(context)?,
                    value: (*value_type).map_node(context)?,
                    self_type,
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
                type_definitions.push(TypeDefinition::External(ExternalType {
                    namespace: namespace_name.clone(),
                    name: name.clone(),
                    self_type,
                }))
            }
            _ => (),
        }
    }
    Ok(type_definitions)
}
