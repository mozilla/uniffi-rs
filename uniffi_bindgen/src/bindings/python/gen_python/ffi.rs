/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Represents an Ffi definition
#[derive(Debug, Clone)]
pub enum FfiDefinition {
    // Scaffolding function exported in the library
    Function(FfiFunction),
    // Function type, for function pointers
    FunctionType(FfiFunctionType),
    // FFI struct definition
    Struct(FfiStruct),
}

/// Represents an "extern C"-style function that will be part of the FFI.
///
/// These can't be declared explicitly in the UDL, but rather, are derived automatically
/// from the high-level interface. Each callable thing in the component API will have a
/// corresponding `FfiFunction` through which it can be invoked, and UniFFI also provides
/// some built-in `FfiFunction` helpers for use in the foreign language bindings.
#[derive(Debug, Clone)]
pub struct FfiFunction {
    pub name: String,
    pub is_async: bool,
    pub arguments: Vec<FfiArgument>,
    pub return_type: Option<String>,
    pub has_rust_call_status_arg: bool,
}

/// Represents an "extern C"-style callback function
///
/// These are defined in the foreign code and passed to Rust as a function pointer.
#[derive(Debug, Clone)]
pub struct FfiFunctionType {
    pub name: String,
    pub arguments: Vec<FfiArgument>,
    pub return_type: Option<String>,
    pub has_rust_call_status_arg: bool,
}

/// Represents a repr(C) struct
#[derive(Debug, Default, Clone)]
pub struct FfiStruct {
    pub name: String,
    pub fields: Vec<FfiField>,
}

/// Represents a field of an [FfiStruct]
#[derive(Debug, Clone)]
pub struct FfiField {
    pub name: String,
    pub ty: String,
}

/// Represents an argument to an FFI function.
///
/// Each argument has a name and a type.
#[derive(Debug, Clone)]
pub struct FfiArgument {
    pub name: String,
    pub ty: String,
}

/// Ffi function to check a checksum for an item in the interface
///
/// Bindings generators should call each of these functions and check that they return the
/// `checksum` value.
#[derive(Debug, Clone)]
pub struct ChecksumCheck {
    pub func: String,
    pub checksum: u16,
}
