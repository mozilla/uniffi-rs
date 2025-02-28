/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add TypeDefinitions for types used in the API as arguments, return types, throws types, etc.
//!
//! Most bindings will need to define FFI converters for these.

use indexmap::IndexSet;

use super::*;

pub fn step(root: &mut Root) -> Result<()> {
    root.visit_mut(|module: &mut Module| {
        // Map types to type definitions to add
        let mut all_types = IndexSet::<Type>::default();
        module.visit(|ty: &Type| {
            all_types.insert(ty.clone());
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
                    module
                        .type_definitions
                        .push(TypeDefinition::Simple(TypeNode {
                            ty,
                            ..TypeNode::empty()
                        }));
                }
                Type::Optional { inner_type } => {
                    module
                        .type_definitions
                        .push(TypeDefinition::Optional(OptionalType {
                            inner: TypeNode {
                                ty: (**inner_type).clone(),
                                ..TypeNode::empty()
                            },
                            self_type: TypeNode {
                                ty,
                                ..TypeNode::empty()
                            },
                        }));
                }
                Type::Sequence { inner_type } => {
                    module
                        .type_definitions
                        .push(TypeDefinition::Sequence(SequenceType {
                            inner: TypeNode {
                                ty: (**inner_type).clone(),
                                ..TypeNode::empty()
                            },
                            self_type: TypeNode {
                                ty,
                                ..TypeNode::empty()
                            },
                        }));
                }
                Type::Map {
                    key_type,
                    value_type,
                } => {
                    module.type_definitions.push(TypeDefinition::Map(MapType {
                        key: TypeNode {
                            ty: (**key_type).clone(),
                            ..TypeNode::empty()
                        },
                        value: TypeNode {
                            ty: (**value_type).clone(),
                            ..TypeNode::empty()
                        },
                        self_type: TypeNode {
                            ty,
                            ..TypeNode::empty()
                        },
                    }));
                }
                _ => (),
            }
        }
    });
    Ok(())
}
