/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// An "object" is an opaque type that is passed around by reference, can
/// have methods called on it, and so on - basically your classic Object Oriented Programming
/// type of deal, except without elaborate inheritance hierarchies. Some can be instantiated.
///
/// In UDL these correspond to the `interface` keyword.
///
/// At the FFI layer, objects are represented by an opaque integer handle and a set of functions
/// a common prefix. The object's constructors are functions that return new objects by handle,
/// and its methods are functions that take a handle as first argument. The foreign language
/// binding code is expected to stitch these functions back together into an appropriate class
/// definition (or that language's equivalent thereof).
///
/// TODO:
///  - maybe "Class" would be a better name than "Object" here?
#[derive(Debug, Clone, Checksum)]
pub struct Object {
    pub(super) name: String,
    /// How this object is implemented in Rust
    pub(super) imp: ObjectImpl,
    pub(super) module_path: String,
    pub(super) remote: bool,
    pub(super) constructors: Vec<Constructor>,
    pub(super) methods: Vec<Method>,
    // The "trait" methods - they have a (presumably "well known") name, and
    // a regular method (albeit with a generated name)
    // XXX - this should really be a HashSet, but not enough transient types support hash to make it worthwhile now.
    pub(super) uniffi_traits: Vec<UniffiTrait>,
    // These are traits described in our CI which this object has declared it implements.
    // This allows foreign bindings to implement things like inheritance or whatever makes sense for them.
    pub(super) trait_impls: Vec<ObjectTraitImplMetadata>,
    // We don't include the FfiFuncs in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from  the very
    //    hash value we're trying to calculate here, so excluding it
    //    avoids a weird circular dependency in the calculation.

    // FFI function to clone a pointer for this object
    #[checksum_ignore]
    pub(super) ffi_func_clone: FfiFunction,
    // FFI function to free a pointer for this object
    #[checksum_ignore]
    pub(super) ffi_func_free: FfiFunction,
    // Ffi function to initialize the foreign callback for trait interfaces
    #[checksum_ignore]
    pub(super) ffi_init_callback: Option<FfiFunction>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

#[derive(Debug, Clone, Checksum)]
pub struct CallbackInterface {
    pub(super) name: String,
    pub(super) module_path: String,
    pub(super) methods: Vec<Method>,
    // We don't include the FFIFunc in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from  the very
    //    hash value we're trying to calculate here, so excluding it
    //    avoids a weird circular dependency in the calculation.
    #[checksum_ignore]
    pub(super) ffi_init_callback: FfiFunction,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

// Represents a constructor for an object type.
//
// In the FFI, this will be a function that returns a pointer to an instance
// of the corresponding object type.
#[derive(Debug, Clone, Checksum)]
pub struct Constructor {
    pub(super) name: String,
    pub(super) object_name: String,
    pub(super) object_module_path: String,
    pub(super) is_async: bool,
    pub(super) arguments: Vec<Argument>,
    // We don't include the FFIFunc in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from  the very
    //    hash value we're trying to calculate here, so excluding it
    //    avoids a weird circular dependency in the calculation.
    #[checksum_ignore]
    pub(super) ffi_func: FfiFunction,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
    pub(super) throws: Option<Type>,
    pub(super) checksum_fn_name: String,
    // Force a checksum value, or we'll fallback to the trait.
    #[checksum_ignore]
    pub(super) checksum: Option<u16>,
}

// Represents an instance method for an object type.
//
// The FFI will represent this as a function whose first/self argument is a
// `FfiType::RustArcPtr` to the instance.
#[derive(Debug, Clone, Checksum)]
pub struct Method {
    pub(super) name: String,
    pub(super) object_name: String,
    pub(super) object_module_path: String,
    pub(super) is_async: bool,
    pub(super) object_impl: ObjectImpl,
    pub(super) arguments: Vec<Argument>,
    pub(super) return_type: Option<Type>,
    // We don't include the FFIFunc in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from  the very
    //    hash value we're trying to calculate here, so excluding it
    //    avoids a weird circular dependency in the calculation.
    #[checksum_ignore]
    pub(super) ffi_func: FfiFunction,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
    pub(super) throws: Option<Type>,
    pub(super) takes_self_by_arc: bool,
    pub(super) checksum_fn_name: String,
    // Force a checksum value, or we'll fallback to the trait.
    #[checksum_ignore]
    pub(super) checksum: Option<u16>,
}
