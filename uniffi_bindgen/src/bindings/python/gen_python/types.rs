/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_internal_macros::AsType;
pub use uniffi_meta::ExternalKind;

use super::*;

/// Type definitions
///
/// Foreign bindings will typically define an FFI converter for these and a class for types like
/// Record, Enum, and Object.
#[derive(Debug, Clone, AsType)]
pub enum TypeDefinition {
    /// Builtin types from the general IR are split into several categories:
    ///     - Simple -- these don't contain any other types
    ///     - Optional, Sequence, and Map -- these need a type definition since Python doesn't have
    ///        generics.  We need to define an FFI converter for each one used in the
    ///        interface.
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
    pub type_name: String,
    pub ffi_converter_name: String,
    /// FFI type for this type
    pub ffi_type: String,
    /// Was this type used as the error half of a result for any function/method/constructor?
    pub is_used_as_error: bool,
}

#[derive(Debug, Clone, AsType)]
pub struct OptionalType {
    pub inner: Type,
    pub self_type: Type,
}

#[derive(Debug, Clone, AsType)]
pub struct SequenceType {
    pub inner: Type,
    pub self_type: Type,
}

#[derive(Debug, Clone, AsType)]
pub struct MapType {
    pub key: Type,
    pub value: Type,
    pub self_type: Type,
}

#[derive(Debug, Clone, AsType)]
pub struct CustomType {
    pub name: String,
    pub config: Option<CustomTypeConfig>,
    pub builtin: Type,
    pub self_type: Type,
}

#[derive(Debug, Clone, AsType)]
pub struct ExternalType {
    pub name: String,
    pub namespace: String,
    pub kind: ExternalKind,
    pub self_type: Type,
}

/// Trait for nodes that are associated with a type
pub trait AsType {
    fn as_type(&self) -> &Type;

    fn is_used_as_error(&self) -> bool {
        self.as_type().is_used_as_error
    }
}

impl AsType for Type {
    fn as_type(&self) -> &Type {
        self
    }
}
impl<T: AsType + ?Sized> AsType for &T {
    fn as_type(&self) -> &Type {
        (*self).as_type()
    }
}
