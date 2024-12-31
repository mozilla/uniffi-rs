/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use uniffi_meta::{ExternalKind, ObjectImpl};

use super::*;

/// Type definitions
///
/// Foreign bindings will typically define an FFI converter for these and a class for types like
/// Record, Enum, and Object.
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    /// Builtin types that don't contain other types
    Simple(Type),
    Optional(OptionalType),
    Sequence(SequenceType),
    Map(MapType),
    Record(Record),
    Enum(Enum),
    Interface(Interface),
    CallbackInterface(CallbackInterface),
    Custom(CustomType),
    External(ExternalType),
}

/// ComponentInterface node that stores a type.
///
/// Nodes like [Record] and [Interface] that represent a type definition, will store one of these in
/// as a `self_type` field
#[derive(Debug, Clone)]
pub struct Type {
    pub kind: uniffi_meta::Type,
    /// FFI type for this type
    pub ffi_type: FfiType,
    /// Was this type used as the error half of a result for any function/method/constructor?
    pub is_used_as_error: bool,
}

/// Optional type
///
/// Bindings generators without generics will probably want to generate a FFI converter for
/// each of these.
#[derive(Debug, Clone)]
pub struct OptionalType {
    pub inner: Type,
    pub self_type: Type,
}

/// Sequence type
///
/// Bindings generators without generics will probably want to generate a FFI converter for
/// each of these.
#[derive(Debug, Clone)]
pub struct SequenceType {
    pub inner: Type,
    pub self_type: Type,
}

/// Map type
///
/// Bindings generators without generics will probably want to generate a FFI converter for
/// each of these.
#[derive(Debug, Clone)]
pub struct MapType {
    pub key: Type,
    pub value: Type,
    pub self_type: Type,
}

/// Definition for custom types
/// This store exactly the same data as Type::Custom, since historically we've stuffed all data in
/// that variant.  Eventually we should consider removing some fields from Type::Custom to make it
/// work more like the other types
#[derive(Debug, Clone)]
pub struct CustomType {
    pub name: String,
    pub module_path: String,
    pub builtin: Type,
    pub self_type: Type,
}

/// Definition for external types
///
/// This store exactly the same data as Type::External, since historically we've stuffed all data in
/// that variant.  Eventually we should consider removing some fields from Type::External to make it
/// work more like the other types
#[derive(Debug, Clone)]
pub struct ExternalType {
    pub name: String,
    pub module_path: String,
    pub namespace: String,
    pub kind: ExternalKind,
    pub self_type: Type,
}

impl TypeDefinition {
    pub fn canonical_name(&self) -> String {
        match self {
            Self::Simple(t) => t.canonical_name(),
            Self::Optional(o) => o.self_type.canonical_name(),
            Self::Sequence(s) => s.self_type.canonical_name(),
            Self::Map(m) => m.self_type.canonical_name(),
            Self::Record(r) => r.self_type.canonical_name(),
            Self::Enum(e) => e.self_type.canonical_name(),
            Self::Interface(i) => i.self_type.canonical_name(),
            Self::CallbackInterface(c) => c.self_type.canonical_name(),
            Self::Custom(c) => c.self_type.canonical_name(),
            Self::External(e) => e.self_type.canonical_name(),
        }
    }
}

impl Type {
    /// Unique, UpperCamelCase name for this type
    ///
    /// This is used in a couple ways:
    ///   - Storing types in a hash map
    ///   - As a base name for helper classes for the type.  For example, bindings will define a
    ///     `UniffiConverter{canonical_name}` class, which handles lifting/lowering the type.
    pub fn canonical_name(&self) -> String {
        canonical_name_for_type(&self.kind)
    }
}

fn canonical_name_for_type(ty: &uniffi_meta::Type) -> String {
    match ty {
        uniffi_meta::Type::UInt8 => "UInt8".to_string(),
        uniffi_meta::Type::Int8 => "Int8".to_string(),
        uniffi_meta::Type::UInt16 => "UInt16".to_string(),
        uniffi_meta::Type::Int16 => "Int16".to_string(),
        uniffi_meta::Type::UInt32 => "UInt32".to_string(),
        uniffi_meta::Type::Int32 => "Int32".to_string(),
        uniffi_meta::Type::UInt64 => "UInt64".to_string(),
        uniffi_meta::Type::Int64 => "Int64".to_string(),
        uniffi_meta::Type::Float32 => "Float32".to_string(),
        uniffi_meta::Type::Float64 => "Float64".to_string(),
        uniffi_meta::Type::Boolean => "Boolean".to_string(),
        uniffi_meta::Type::String => "String".to_string(),
        uniffi_meta::Type::Bytes => "Bytes".to_string(),
        uniffi_meta::Type::Timestamp => "Timestamp".to_string(),
        uniffi_meta::Type::Duration => "Duration".to_string(),
        uniffi_meta::Type::Object { name, .. }
        | uniffi_meta::Type::Record { name, .. }
        | uniffi_meta::Type::Enum { name, .. }
        | uniffi_meta::Type::CallbackInterface { name, .. }
        | uniffi_meta::Type::Custom { name, .. }
        | uniffi_meta::Type::External { name, .. } => format!("Type{name}"),
        uniffi_meta::Type::Optional { inner_type } => {
            format!("Optional{}", canonical_name_for_type(inner_type))
        }
        uniffi_meta::Type::Sequence { inner_type } => {
            format!("Sequence{}", canonical_name_for_type(inner_type))
        }
        // Note: this is currently guaranteed to be unique because keys can only be primitive
        // types.  If we allowed user-defined types, there would be potential collisions.  For
        // example "MapTypeFooTypeTypeBar" could be "Foo" -> "TypeBar" or "FooType" -> "Bar".
        uniffi_meta::Type::Map {
            key_type,
            value_type,
        } => format!(
            "Map{}{}",
            canonical_name_for_type(key_type),
            canonical_name_for_type(value_type),
        ),
    }
}
