/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! TypeNode::is_used_as_error field
//!
//! Bindings often treat error types differently, for example by adding `Exception` to their names.

use anyhow::bail;
use indexmap::IndexSet;

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let mut used_as_error = IndexSet::<String>::default();
    module.try_visit(|callable: &Callable| {
        if let Some(ty) = &callable.throws_type.ty {
            let type_name = match ty.ty.name() {
                Some(name) => name.to_string(),
                _ => bail!("Invalid throws type: {:?}", ty.ty),
            };
            used_as_error.insert(type_name);
        }
        Ok(())
    })?;
    module.visit_mut(|type_node: &mut TypeNode| {
        if let Some(name) = type_node.ty.name() {
            type_node.is_used_as_error = used_as_error.contains(name);
        }
        type_node.canonical_name = canonical_name(&type_node.ty);
    });
    Ok(())
}

fn canonical_name(ty: &Type) -> String {
    match ty {
        Type::UInt8 => "UInt8".to_string(),
        Type::Int8 => "Int8".to_string(),
        Type::UInt16 => "UInt16".to_string(),
        Type::Int16 => "Int16".to_string(),
        Type::UInt32 => "UInt32".to_string(),
        Type::Int32 => "Int32".to_string(),
        Type::UInt64 => "UInt64".to_string(),
        Type::Int64 => "Int64".to_string(),
        Type::Float32 => "Float32".to_string(),
        Type::Float64 => "Float64".to_string(),
        Type::Boolean => "Boolean".to_string(),
        Type::String => "String".to_string(),
        Type::Bytes => "Bytes".to_string(),
        Type::Timestamp => "Timestamp".to_string(),
        Type::Duration => "Duration".to_string(),
        Type::Interface { name, .. }
        | Type::Record { name, .. }
        | Type::Enum { name, .. }
        | Type::CallbackInterface { name, .. }
        | Type::Custom { name, .. } => format!("Type{name}"),
        Type::Optional { inner_type } => {
            format!("Optional{}", canonical_name(inner_type))
        }
        Type::Sequence { inner_type } => {
            format!("Sequence{}", canonical_name(inner_type))
        }
        // Note: this is currently guaranteed to be unique because keys can only be primitive
        // types.  If we allowed user-defined types, there would be potential collisions.  For
        // example "MapTypeFooTypeTypeBar" could be "Foo" -> "TypeBar" or "FooType" -> "Bar".
        Type::Map {
            key_type,
            value_type,
        } => format!(
            "Map{}{}",
            canonical_name(key_type),
            canonical_name(value_type),
        ),
    }
}
