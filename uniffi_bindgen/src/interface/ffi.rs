/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! # Low-level typesystem for the FFI layer of a component interface.
//!
//! This module provides the "FFI-level" typesystem of a UniFFI Rust Component, that is,
//! the C-style functions and structs and primitive datatypes that are used to interface
//! between the Rust component code and the foreign-language bindings.
//!
//! These types are purely an implementation detail of UniFFI, so consumers shouldn't
//! need to know about them. But as a developer working on UniFFI itself, you're likely
//! to spend a lot of time thinking about how these low-level types are used to represent
//! the higher-level "interface types" from the [`super::types::Type`] enum.
/// Represents the restricted set of low-level types that can be used to construct
/// the C-style FFI layer between a rust component and its foreign language bindings.
///
/// For the types that involve memory allocation, we make a distinction between
/// "owned" types (the recipient must free it, or pass it to someone else) and
/// "borrowed" types (the sender must keep it alive for the duration of the call).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FFIType {
    // N.B. there are no booleans at this layer, since they cause problems for JNA.
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
    /// A `char*` pointer belonging to a rust-owned CString.
    /// If you've got one of these, you must call the appropriate rust function to free it.
    /// This is currently only used for error messages, and may go away in future.
    RustCString,
    /// A byte buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    RustBuffer,
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
    ForeignBytes,
    /// An error struct, containing a numberic error code and char* pointer to error string.
    /// The string is owned by rust and allocated on the rust heap, and must be freed by
    /// passing it to the appropriate `string_free` FFI function.
    RustError,
    /// A pointer to a single function in to the foreign language.
    /// This function contains all the machinery to make callbacks work on the foreign language side.
    ForeignCallback,
    // TODO: you can imagine a richer structural typesystem here, e.g. `Ref<String>` or something.
    // We don't need that yet and it's possible we never will, so it isn't here for now.
}

/// Represents an "extern C"-style function that will be part of the FFI.
///
/// These can't be declared explicitly in the UDL, but rather, are derived automatically
/// from the high-level interface. Each callable thing in the component API will have a
/// corresponding `FFIFunction` through which it can be invoked, and UniFFI also provides
/// some built-in `FFIFunction` helpers for use in the foreign language bindings.
#[derive(Debug, Default, Clone)]
pub struct FFIFunction {
    pub(super) name: String,
    pub(super) arguments: Vec<FFIArgument>,
    pub(super) return_type: Option<FFIType>,
}

impl FFIFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> Vec<&FFIArgument> {
        self.arguments.iter().collect()
    }
    pub fn return_type(&self) -> Option<&FFIType> {
        self.return_type.as_ref()
    }
}

/// Represents an argument to an FFI function.
///
/// Each argument has a name and a type.
#[derive(Debug, Clone)]
pub struct FFIArgument {
    pub(super) name: String,
    pub(super) type_: FFIType,
}

impl FFIArgument {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> FFIType {
        self.type_.clone()
    }
}

#[cfg(test)]
mod test {
    // There's not really much to test here to be honest,
    // it's mostly type declarations.
}
