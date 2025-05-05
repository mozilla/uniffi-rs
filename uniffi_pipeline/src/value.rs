/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{FromValueError, Node, Result};
use std::{collections::HashMap, fmt};

/// Represents the value of a node
///
/// This is a loosely typed data structure similar to `serde_json::Value`.
/// Nodes can be converted to/from `Value` which is what allows nodes to automatically be converted
/// from one IR to another.
pub enum Value {
    UInt8(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    UInt32(u32),
    Int32(i32),
    UInt64(u64),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bool(bool),
    Option(Option<Box<Value>>),
    Vec(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Struct {
        type_name: &'static str,
        fields: HashMap<&'static str, Value>,
    },
    Variant {
        type_name: &'static str,
        variant_name: &'static str,
        fields: HashMap<&'static str, Value>,
    },
}

impl Value {
    pub fn try_into_node<T: Node>(self) -> Result<T, FromValueError> {
        T::try_from_value(self)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UInt8(val) => write!(f, "{val}u8"),
            Self::Int8(val) => write!(f, "{val}i8"),
            Self::UInt16(val) => write!(f, "{val}u16"),
            Self::Int16(val) => write!(f, "{val}i16"),
            Self::UInt32(val) => write!(f, "{val}u32"),
            Self::Int32(val) => write!(f, "{val}i32"),
            Self::UInt64(val) => write!(f, "{val}u64"),
            Self::Int64(val) => write!(f, "{val}i64"),
            Self::Float32(val) => write!(f, "{val}f32"),
            Self::Float64(val) => write!(f, "{val}f64"),
            Self::String(val) => write!(f, "{val:?}"),
            Self::Bool(val) => write!(f, "{val:?}"),
            Self::Option(val) => match val {
                Some(v) => f.debug_tuple("Some").field(v).finish(),
                None => f.debug_tuple("None").finish(),
            },
            Self::Vec(values) => f.debug_list().entries(values).finish(),
            Self::Map(values) => f
                .debug_map()
                .entries(values.iter().map(|(k, v)| (k, v)))
                .finish(),
            Self::Set(values) => f.debug_set().entries(values.iter()).finish(),
            Self::Struct { type_name, fields } => {
                let mut f = f.debug_struct(type_name);
                for (name, val) in fields {
                    f.field(name, val);
                }
                f.finish()
            }
            Self::Variant {
                type_name,
                variant_name,
                fields,
            } => {
                let mut f = f.debug_struct(&format!("{type_name}::{variant_name}"));
                for (name, val) in fields {
                    f.field(name, val);
                }
                f.finish()
            }
        }
    }
}
