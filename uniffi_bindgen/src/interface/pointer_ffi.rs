/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI definitions for the pointer FFI
//!
//! These exist in parallel with the normal/legacy FFI.  If the user enables the `pointer-ffi`
//! feature, then both versions of the FFI will be generated.  The pointer FFI symbols are prefixed
//! with `uniffi_ptr_` to avoid any conflicts.

use super::{ComponentInterface, UniffiTrait};

pub enum PointerFfiDefinition {
    Function(PointerFfiFunction),
}

/// FFI function for the pointer FFI
pub struct PointerFfiFunction {
    pub name: String,
}

pub fn ffi_definitions(ci: &ComponentInterface) -> impl Iterator<Item = PointerFfiDefinition> + '_ {
    let namespace = ci.ffi_namespace();
    // Builtin FFI methods
    [
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rustbuffer_alloc")),
        PointerFfiDefinition::func(format!("uniffi_ptr_{namespace}_rustbuffer_free")),
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
    fn func(name: impl Into<String>) -> Self {
        Self::Function(PointerFfiFunction { name: name.into() })
    }
}

impl ComponentInterface {
    pub fn pointer_ffi_rustbuffer_alloc(&self) -> String {
        format!("uniffi_ptr_{}_rustbuffer_alloc", self.ffi_namespace())
    }

    pub fn pointer_ffi_rustbuffer_free(&self) -> String {
        format!("uniffi_ptr_{}_rustbuffer_free", self.ffi_namespace())
    }
}
