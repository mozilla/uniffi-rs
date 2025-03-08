/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::vtable as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    struct TypeNode {
        /// Python type name
        +type_name: String,
    }

    fn add_typenode_type_name(type_node: &prev::TypeNode) -> String {
        type_name(&type_node.ty)
    }
}

fn type_name(ty: &prev::Type) -> String {
    match ty {
        prev::Type::Boolean => "bool".to_string(),
        prev::Type::String => "str".to_string(),
        prev::Type::Bytes => "bytes".to_string(),
        prev::Type::Int8 => "int".to_string(),
        prev::Type::Int16
        | prev::Type::Int32
        | prev::Type::Int64
        | prev::Type::UInt8
        | prev::Type::UInt16
        | prev::Type::UInt32
        | prev::Type::UInt64 => "int".to_string(),
        prev::Type::Duration => "Duration".to_string(),
        prev::Type::Timestamp => "Timestamp".to_string(),
        prev::Type::Float32 | prev::Type::Float64 => "float".to_string(),
        prev::Type::Interface { name, .. }
        | prev::Type::Record { name, .. }
        | prev::Type::Enum { name, .. }
        | prev::Type::CallbackInterface { name, .. }
        | prev::Type::Custom { name, .. } => name.to_string(),
        prev::Type::Optional { inner_type } => {
            format!("typing.Optional[{}]", type_name(inner_type))
        }
        prev::Type::Sequence { inner_type } => {
            format!("typing.List[{}]", type_name(inner_type))
        }
        prev::Type::Map {
            key_type,
            value_type,
        } => format!("dict[{}, {}]", type_name(key_type), type_name(value_type)),
    }
}
