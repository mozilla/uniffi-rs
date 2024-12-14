/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_internal_macros::{AsCallable, AsType};

use super::{AsCallable, AsType, Callable, Type};

#[derive(Debug, Clone, AsType)]
pub struct Interface {
    pub name: String,
    pub protocol_name: String,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub had_async_constructor: bool,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub base_classes: Vec<String>,
    pub vtable: Option<VTable>,
    pub ffi_clone: String,
    pub ffi_free: String,
    pub docstring: Option<String>,
    pub self_type: Type,
}

/// Rust traits we support generating helper methods for.
#[derive(Clone, Debug)]
pub enum UniffiTrait {
    Debug { fmt: Method },
    Display { fmt: Method },
    Eq { eq: Method, ne: Method },
    Hash { hash: Method },
}

/// Interface implemented on the foreign side of the FFI
#[derive(Debug, Clone, AsType)]
pub struct CallbackInterface {
    pub name: String,
    pub methods: Vec<Method>,
    pub vtable: VTable,
    pub docstring: Option<String>,
    pub self_type: Type,
}

/// VTable for a callback / trait interface
#[derive(Clone, Debug)]
pub struct VTable {
    /// Initially the name of the CallbalkInterface or Interface associated with this VTable.
    /// Languages can change to the name of the VTable item they generate.
    pub name: String,
    /// FFI type for the VTable.
    pub ffi_type: String,
    // Ffi function to initialize the foreign callback for trait interfaces
    pub ffi_init_callback: String,
    pub methods: Vec<VTableMethod>,
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a pointer to an instance
// of the corresponding object type.
#[derive(Debug, Clone, AsCallable)]
pub struct Constructor {
    pub name: String,
    pub primary: bool,
    pub interface: Type,
    pub callable: Callable,
    pub docstring: Option<String>,
}

/// Represents an instance method for an object type.
#[derive(Debug, Clone, AsCallable)]
pub struct Method {
    pub name: String,
    pub interface: Type,
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, AsCallable)]
pub struct VTableMethod {
    pub name: String,
    pub ffi_type: String,
    pub default_return_value: String,
    pub callable: Callable,
}

impl Interface {
    pub fn has_callback_interface(&self) -> bool {
        self.vtable.is_some()
    }

    pub fn primary_constructor(&self) -> Option<Constructor> {
        self.constructors.iter().find(|c| c.primary).cloned()
    }

    pub fn alternate_constructors(&self) -> impl Iterator<Item = &Constructor> {
        self.constructors.iter().filter(|c| !c.primary)
    }
}

impl UniffiTrait {
    pub fn methods(&self) -> Vec<&Method> {
        match self {
            UniffiTrait::Debug { fmt } | UniffiTrait::Display { fmt } => vec![fmt],
            UniffiTrait::Eq { eq, ne } => vec![eq, ne],
            UniffiTrait::Hash { hash } => vec![hash],
        }
    }
}
