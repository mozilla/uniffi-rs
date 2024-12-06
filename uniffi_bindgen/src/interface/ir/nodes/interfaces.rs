/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta::{ObjectImpl, ObjectTraitImplMetadata};

use super::{
    Argument, AsyncData, FfiFunctionRef, FfiType, LanguageData, ReturnType, ThrowsType, Type,
};

/// An exported interface.
///
/// These are passed around by reference, can have methods called on it, and so on - basically your
/// classic Object Oriented Programming type of deal, except without elaborate inheritance
/// hierarchies.
///
/// There are currently three styles of interfaces:
///
///   - Object: A single Rust type with exposed methods.
///   - Trait: A Rust trait that can be implemented by multiple types.
///   - Trait with foreign: A Rust trait that can also be implemented by the foreign side.  The
///     generated bindings define and register a vtable for these.
///
/// The `ObjectImpl` enum distinguishes between these three types, although with different names.
#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    /// How this object is implemented in Rust
    pub imp: ObjectImpl,
    pub module_path: String,
    pub remote: bool,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    // The "trait" methods - they have a (presumably "well known") name, and
    // a regular method (albeit with a generated name)
    // XXX - this should really be a HashSet, but not enough transient types support hash to make it worthwhile now.
    pub uniffi_traits: Vec<UniffiTrait>,
    // These are traits described in our CI which this object has declared it implements.
    // This allows foreign bindings to implement things like inheritance or whatever makes sense for them.
    pub trait_impls: Vec<ObjectTraitImplMetadata>,
    // VTable for trait interfaces
    pub vtable: Option<VTable>,
    // FFI function to clone a pointer for this object
    pub ffi_clone: FfiFunctionRef,
    // FFI function to free a pointer for this object
    pub ffi_free: FfiFunctionRef,
    pub docstring: Option<String>,
    pub self_type: Type,
    pub lang_data: LanguageData,
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
#[derive(Debug, Clone)]
pub struct CallbackInterface {
    pub name: String,
    pub module_path: String,
    pub methods: Vec<Method>,
    pub vtable: VTable,
    pub docstring: Option<String>,
    pub self_type: Type,
    pub lang_data: LanguageData,
}

/// VTable for a callback / trait interface
#[derive(Clone, Debug)]
pub struct VTable {
    /// Initially the name of the CallbalkInterface or Interface associated with this VTable.
    /// Languages can change to the name of the VTable item they generate.
    pub name: String,
    /// FFI type for the VTable.
    pub ffi_type: FfiType,
    // Ffi function to initialize the foreign callback for trait interfaces
    pub ffi_init_callback: FfiFunctionRef,
    pub methods: Vec<Method>,
    /// FfiFunctionType for the VTable field for each method
    pub method_ffi_types: Vec<FfiType>,
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a pointer to an instance
// of the corresponding object type.
#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub primary: bool,
    pub interface: Type,
    pub object_module_path: String,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
    pub ffi_func: FfiFunctionRef,
    pub docstring: Option<String>,
    pub throws_type: ThrowsType,
    pub checksum_func: FfiFunctionRef,
    pub checksum: u16,
    pub lang_data: LanguageData,
}

/// Represents an instance method for an object type.
///
/// The FFI will represent this as a function whose first/self argument is a
/// `FfiType::RustArcPtr` to the instance.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub interface: Type,
    pub object_module_path: String,
    pub async_data: Option<AsyncData>,
    pub object_impl: ObjectImpl,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
    pub ffi_func: FfiFunctionRef,
    pub docstring: Option<String>,
    pub throws_type: ThrowsType,
    pub checksum_func: FfiFunctionRef,
    pub checksum: u16,
    pub lang_data: LanguageData,
}

impl Interface {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }

    pub fn has_async_method(&self) -> bool {
        self.methods.iter().any(|m| m.is_async())
    }

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

impl CallbackInterface {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }

    pub fn has_async_method(&self) -> bool {
        self.methods.iter().any(|m| m.is_async())
    }
}

impl VTable {
    pub fn methods_and_ffi_types(&self) -> impl Iterator<Item = (&Method, &FfiType)> {
        self.methods.iter().zip(self.method_ffi_types.iter())
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

impl Constructor {
    pub fn is_async(&self) -> bool {
        self.async_data.is_some()
    }

    pub fn is_sync(&self) -> bool {
        !self.is_async()
    }
}

impl Method {
    pub fn is_async(&self) -> bool {
        self.async_data.is_some()
    }

    pub fn is_sync(&self) -> bool {
        !self.is_async()
    }
}
