/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// An Ffi definition
#[derive(Debug, Clone)]
pub enum FfiDefinition {
    Function(FfiFunction),
    CallbackFunction(FfiCallbackFunction),
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
    pub(super) name: String,
    pub(super) is_async: bool,
    pub(super) arguments: Vec<FfiArgument>,
    pub(super) return_type: Option<FfiType>,
    pub(super) has_rust_call_status_arg: bool,
    /// Used by C# generator to differentiate the free function and call it with void*
    /// instead of C# `SafeHandle` type. See <https://github.com/mozilla/uniffi-rs/pull/1488>.
    pub(super) is_object_free_function: bool,
}

/// Represents an "extern C"-style callback function
///
/// These are defined in the foreign code and passed to Rust as a function pointer.
#[derive(Debug, Default, Clone)]
pub struct FfiCallbackFunction {
    // Name for this function type. This matches the value inside `FfiType::Callback`
    pub(super) name: String,
    pub(super) arguments: Vec<FfiArgument>,
    pub(super) return_type: Option<FfiType>,
    pub(super) has_rust_call_status_arg: bool,
}

/// Represents a repr(C) struct used in the FFI
#[derive(Debug, Default, Clone)]
pub struct FfiStruct {
    pub(super) name: String,
    pub(super) fields: Vec<FfiField>,
}

/// Represents a field of an [FfiStruct]
#[derive(Debug, Clone)]
pub struct FfiField {
    pub(super) name: String,
    pub(super) type_: FfiType,
}

/// Represents an argument to an FFI function.
///
/// Each argument has a name and a type.
#[derive(Debug, Clone)]
pub struct FfiArgument {
    pub(super) name: String,
    pub(super) type_: FfiType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FfiType {
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
    /// A `*const c_void` pointer to a rust-owned `Arc<T>`.
    /// If you've got one of these, you must call the appropriate rust function to free it.
    /// The templates will generate a unique `free` function for each T.
    /// The inner string references the name of the `T` type.
    RustArcPtr(String),
    /// A byte buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    /// If the inner option is Some, it is the name of the external type. The bindings may need
    /// to use this name to import the correct RustBuffer for that type.
    RustBuffer(Option<ExternalFfiMetadata>),
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
    ForeignBytes,
    /// Pointer to a callback function.  The inner value which matches one of the callback
    /// definitions in [crate::ComponentInterface::ffi_definitions].
    Callback(String),
    /// Pointer to a FFI struct (e.g. a VTable).  The inner value matches one of the struct
    /// definitions in [crate::ComponentInterface::ffi_definitions].
    Struct(String),
    /// Opaque 64-bit handle
    ///
    /// These are used to pass objects across the FFI.
    Handle,
    RustCallStatus,
    /// Const pointer to an FfiType.
    Reference(Box<FfiType>),
    /// Mutable pointer to an FfiType.
    MutReference(Box<FfiType>),
    /// Opaque pointer
    VoidPointer,
}
