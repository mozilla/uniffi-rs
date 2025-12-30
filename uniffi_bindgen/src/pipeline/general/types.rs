/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn canonical_name(ty: &Type) -> String {
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
        | Type::CallbackInterface { name, .. }
        | Type::Record { name, .. }
        | Type::Enum { name, .. }
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

pub fn map_type(mut ty: Type, context: &Context) -> Result<Type> {
    Ok(match ty {
        // Map names for top-level types
        Type::Record {
            ref namespace,
            ref mut name,
            ..
        }
        | Type::Enum {
            ref namespace,
            ref mut name,
            ..
        }
        | Type::Interface {
            ref namespace,
            ref mut name,
            ..
        }
        | Type::CallbackInterface {
            ref namespace,
            ref mut name,
            ..
        }
        | Type::Custom {
            ref namespace,
            ref mut name,
            ..
        } => {
            *name = rename::type_(namespace, name.clone(), context)?;
            ty
        }
        // Map inner types
        Type::Optional { inner_type } => Type::Optional {
            inner_type: Box::new(map_type(*inner_type, context)?),
        },
        Type::Sequence { inner_type } => Type::Sequence {
            inner_type: Box::new(map_type(*inner_type, context)?),
        },
        Type::Map {
            key_type,
            value_type,
        } => Type::Map {
            key_type: Box::new(map_type(*key_type, context)?),
            value_type: Box::new(map_type(*value_type, context)?),
        },
        // All other types can be returned unchanged
        _ => ty,
    })
}
