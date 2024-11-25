/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Represents a standalone function.
///
/// Each `Function` corresponds to a standalone function in the rust module,
/// and has a corresponding standalone function in the foreign language bindings.
///
/// In the FFI, this will be a standalone function with appropriately lowered types.
#[derive(Debug, Clone, Checksum)]
pub struct Function {
    pub(super) name: String,
    pub(super) module_path: String,
    pub(super) is_async: bool,
    pub(super) arguments: Vec<Argument>,
    pub(super) return_type: Option<Type>,
    // We don't include the FFIFunc in the hash calculation, because:
    //  - it is entirely determined by the other fields,
    //    so excluding it is safe.
    //  - its `name` property includes a checksum derived from the very
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

/// Represents an argument to a function/constructor/method call.
///
/// Each argument has a name and a type, along with some optional metadata.
#[derive(Debug, Clone, Checksum)]
pub struct Argument {
    pub(super) name: String,
    pub(super) type_: Type,
    pub(super) by_ref: bool,
    pub(super) optional: bool,
    pub(super) default: Option<Literal>,
}
