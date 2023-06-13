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
    RustBuffer(Option<String>),
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
    ForeignBytes,
    /// Pointer to a callback function that handles all callbacks on the foreign language side.
    ForeignCallback,
    /// Pointer-sized opaque handle that represents a foreign executor.  Foreign bindings can
    /// either use an actual pointer or a usized integer.
    ForeignExecutorHandle,
    /// Pointer to the callback function that's invoked to schedule calls with a ForeignExecutor
    ForeignExecutorCallback,
    /// Pointer to a callback function to complete an async Rust function
    FutureCallback {
        /// Note: `return_type` is not optional because we have a void callback parameter like we
        /// can have a void return.  Instead, we use `UInt8` as a placeholder value.
        return_type: Box<FfiType>,
    },
    /// Opaque pointer passed to the FutureCallback
    FutureCallbackData,
    // TODO: you can imagine a richer structural typesystem here, e.g. `Ref<String>` or something.
    // We don't need that yet and it's possible we never will, so it isn't here for now.
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

impl FfiFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_async(&self) -> bool {
        self.is_async
    }

    pub fn arguments(&self) -> Vec<&FfiArgument> {
        self.arguments.iter().collect()
    }

    pub fn return_type(&self) -> Option<&FfiType> {
        self.return_type.as_ref()
    }

    pub fn has_rust_call_status_arg(&self) -> bool {
        self.has_rust_call_status_arg
    }

    pub fn is_object_free_function(&self) -> bool {
        self.is_object_free_function
    }

    pub fn init(
        &mut self,
        return_type: Option<FfiType>,
        args: impl IntoIterator<Item = FfiArgument>,
    ) {
        self.arguments = args.into_iter().collect();
        if self.is_async() {
            self.arguments.extend([
                // Used to schedule polls
                FfiArgument {
                    name: "uniffi_executor".into(),
                    type_: FfiType::ForeignExecutorHandle,
                },
                // Invoked when the future is ready
                FfiArgument {
                    name: "uniffi_callback".into(),
                    type_: FfiType::FutureCallback {
                        return_type: Box::new(return_type.unwrap_or(FfiType::UInt8)),
                    },
                },
                // Data pointer passed to the callback
                FfiArgument {
                    name: "uniffi_callback_data".into(),
                    type_: FfiType::FutureCallbackData,
                },
            ]);
            // Async scaffolding functions never return values.  Instead, the callback is invoked
            // when the Future is ready.
            self.return_type = None;
        } else {
            self.return_type = return_type;
        }
    }
}

impl Default for FfiFunction {
    fn default() -> Self {
        Self {
            name: "".into(),
            is_async: false,
            arguments: Vec::new(),
            return_type: None,
            has_rust_call_status_arg: true,
            is_object_free_function: false,
        }
    }
}

/// Represents an argument to an FFI function.
///
/// Each argument has a name and a type.
#[derive(Debug, Clone)]
pub struct FfiArgument {
    pub(super) name: String,
    pub(super) type_: FfiType,
}

impl FfiArgument {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn type_(&self) -> FfiType {
        self.type_.clone()
    }
}

#[cfg(test)]
mod test {
    // There's not really much to test here to be honest,
    // it's mostly type declarations.
}
