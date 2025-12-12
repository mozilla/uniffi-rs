/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI definitions for the pointer FFI
//!
//! These exist in parallel with the normal/legacy FFI.  If the user enables the `pointer-ffi`
//! feature, then both versions of the FFI will be generated.  The pointer FFI symbols are prefixed
//! with `uniffi_ptr_` to avoid any conflicts.

use super::{ComponentInterface, UniffiTrait};

#[derive(Debug, Clone)]
pub enum PointerFfiDefinition {
    Callback(PointerFfiCallbackFunction),
    Function(PointerFfiFunction),
}

/// FFI function for the pointer FFI
#[derive(Debug, Clone)]
pub struct PointerFfiFunction {
    pub name: String,
    pub kind: FfiFunctionKind,
}

/// Identifies the FFI function type
///
/// This is used in the pointer FFI to determine the argument types.
#[derive(Debug, Clone, Copy)]
pub enum FfiFunctionKind {
    /// Normal function, inputs a buffer pointer
    Normal,
    /// Rust future poll, inputs a buffer pointer plus a function pointer for the continuation
    /// function
    RustFuturePoll,
}

/// FFI function for the pointer FFI
#[derive(Debug, Clone)]
pub struct PointerFfiCallbackFunction {
    pub name: String,
    pub kind: FfiCallbackFunctionKind,
}

#[derive(Debug, Clone, Copy)]
pub enum FfiCallbackFunctionKind {
    /// Callback for the `rust_future_poll` function
    RustFutureContinutation,
}

pub fn ffi_definitions(ci: &ComponentInterface) -> impl Iterator<Item = PointerFfiDefinition> + '_ {
    let namespace = ci.ffi_namespace();
    // Builtin FFI methods
    [
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rustbuffer_alloc")),
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rustbuffer_free")),
        PointerFfiDefinition::rust_future_poll(format!("uniffi_ptr_{namespace}_rust_future_poll")),
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rust_future_cancel")),
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rust_future_complete")),
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rust_future_free")),
        PointerFfiDefinition::rust_future_continuation("RustFutureContinuationCallback"),
    ]
    .into_iter()
    // Functions
    .chain(
        ci.function_definitions()
            .iter()
            .map(|f| PointerFfiDefinition::func(f.ffi_func().pointer_ffi_name())),
    )
    // Constructors
    .chain(ci.object_definitions().iter().flat_map(|o| {
        o.constructors()
            .into_iter()
            .map(|c| PointerFfiDefinition::func(c.ffi_func().pointer_ffi_name()))
    }))
    // Methods
    .chain(
        ci.enum_definitions()
            .iter()
            .flat_map(|e| e.methods())
            .chain(ci.record_definitions().iter().flat_map(|r| r.methods()))
            .chain(ci.object_definitions().iter().flat_map(|o| o.methods()))
            .map(|m| PointerFfiDefinition::func(m.ffi_func().pointer_ffi_name())),
    )
    // UniFFI trait methods
    .chain(
        ci.enum_definitions()
            .iter()
            .flat_map(|e| e.uniffi_traits())
            .chain(
                ci.record_definitions()
                    .iter()
                    .flat_map(|r| r.uniffi_traits()),
            )
            .chain(
                ci.object_definitions()
                    .iter()
                    .flat_map(|o| o.uniffi_traits()),
            )
            .flat_map(|ut| match ut {
                UniffiTrait::Display { fmt: m }
                | UniffiTrait::Debug { fmt: m }
                | UniffiTrait::Hash { hash: m }
                | UniffiTrait::Ord { cmp: m } => vec![m],
                UniffiTrait::Eq { eq, ne } => vec![eq, ne],
            })
            .map(|m| PointerFfiDefinition::func(m.ffi_func().pointer_ffi_name())),
    )
}

impl PointerFfiDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::Callback(cb) => &cb.name,
            Self::Function(f) => &f.name,
        }
    }

    fn func(name: impl Into<String>) -> Self {
        Self::Function(PointerFfiFunction {
            name: name.into(),
            kind: FfiFunctionKind::Normal,
        })
    }

    fn rust_future_poll(name: impl Into<String>) -> Self {
        Self::Function(PointerFfiFunction {
            name: name.into(),
            kind: FfiFunctionKind::RustFuturePoll,
        })
    }

    fn rust_future_continuation(name: impl Into<String>) -> Self {
        Self::Callback(PointerFfiCallbackFunction {
            name: name.into(),
            kind: FfiCallbackFunctionKind::RustFutureContinutation,
        })
    }
}

impl ComponentInterface {
    pub fn pointer_ffi_rustbuffer_alloc(&self) -> String {
        format!("uniffi_ptr_{}_rustbuffer_alloc", self.ffi_namespace())
    }

    pub fn pointer_ffi_rustbuffer_free(&self) -> String {
        format!("uniffi_ptr_{}_rustbuffer_free", self.ffi_namespace())
    }

    pub fn pointer_ffi_rust_future_poll(&self) -> String {
        format!("uniffi_ptr_{}_rust_future_poll", self.ffi_namespace())
    }

    pub fn pointer_ffi_rust_future_complete(&self) -> String {
        format!("uniffi_ptr_{}_rust_future_complete", self.ffi_namespace())
    }

    pub fn pointer_ffi_rust_future_cancel(&self) -> String {
        format!("uniffi_ptr_{}_rust_future_cancel", self.ffi_namespace())
    }

    pub fn pointer_ffi_rust_future_free(&self) -> String {
        format!("uniffi_ptr_{}_rust_future_free", self.ffi_namespace())
    }
}
