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
    Builtin(Type),
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
    pub kind: TypeKind,
    /// FFI type for this type
    pub ffi_type: FfiType,
    /// Was this type used as the error half of a result for any function/method/constructor?
    pub is_used_as_error: bool,
    pub lang_data: LanguageData,
}

impl Type {
    /// Unique, UpperCamelCase name for this type
    ///
    /// This is used in a couple ways:
    ///   - Storing types in a hash map
    ///   - As a base name for helper classes for the type.  For example, bindings will define a
    ///     `UniffiConverter{canonical_name}` class, which handles lifting/lowering the type.
    pub fn canonical_name(&self) -> String {
        match &self.kind {
            TypeKind::UInt8 => "UInt8".to_string(),
            TypeKind::Int8 => "Int8".to_string(),
            TypeKind::UInt16 => "UInt16".to_string(),
            TypeKind::Int16 => "Int16".to_string(),
            TypeKind::UInt32 => "UInt32".to_string(),
            TypeKind::Int32 => "Int32".to_string(),
            TypeKind::UInt64 => "UInt64".to_string(),
            TypeKind::Int64 => "Int64".to_string(),
            TypeKind::Float32 => "Float32".to_string(),
            TypeKind::Float64 => "Float64".to_string(),
            TypeKind::Boolean => "Boolean".to_string(),
            TypeKind::String => "String".to_string(),
            TypeKind::Bytes => "Bytes".to_string(),
            TypeKind::Timestamp => "Timestamp".to_string(),
            TypeKind::Duration => "Duration".to_string(),
            TypeKind::Interface { name, .. }
            | TypeKind::Record { name, .. }
            | TypeKind::Enum { name, .. }
            | TypeKind::CallbackInterface { name, .. }
            | TypeKind::Custom { name, .. }
            | TypeKind::External { name, .. } => format!("Type{name}"),
            TypeKind::Optional { inner_type } => format!("Optional{}", inner_type.canonical_name()),
            TypeKind::Sequence { inner_type } => format!("Sequence{}", inner_type.canonical_name()),
            // Note: this is currently guaranteed to be unique because keys can only be primitive
            // types.  If we allowed user-defined types, there would be potential collisions.  For
            // example "MapTypeFooTypeTypeBar" could be "Foo" -> "TypeBar" or "FooType" -> "Bar".
            TypeKind::Map {
                key_type,
                value_type,
            } => format!(
                "Map{}{}",
                key_type.canonical_name(),
                value_type.canonical_name()
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    // Primitive types.
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
    Bytes,
    Timestamp,
    Duration,
    Interface {
        // The module path to the object
        module_path: String,
        // The name in the "type universe"
        name: String,
        // How the object is implemented.
        imp: ObjectImpl,
    },
    // Types defined in the component API, each of which has a string name.
    Record {
        module_path: String,
        name: String,
    },
    Enum {
        module_path: String,
        name: String,
    },
    CallbackInterface {
        module_path: String,
        name: String,
    },
    // Structurally recursive types.
    Optional {
        inner_type: Box<Type>,
    },
    Sequence {
        inner_type: Box<Type>,
    },
    Map {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    // An FfiConverter we `use` from an external crate
    External {
        module_path: String,
        name: String,
        namespace: String,
        kind: ExternalKind,
    },
    // Custom type on the scaffolding side
    Custom {
        module_path: String,
        name: String,
        builtin: Box<Type>,
    },
}
