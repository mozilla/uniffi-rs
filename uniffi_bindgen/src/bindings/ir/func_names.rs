/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Names for internal functions
use crate::interface::Type;
use heck::ToSnakeCase;

/// Lift a value from the FFI
pub fn lift(type_: &Type) -> String {
    format!("uniffi_lift_{}", type_.canonical_name().to_snake_case())
}

/// Lower a value for the FFI
pub fn lower(type_: &Type) -> String {
    format!("uniffi_lower_{}", type_.canonical_name().to_snake_case())
}

/// Calculate the size of the RustBuffer needed to fit this type
pub fn allocation_size(type_: &Type) -> String {
    format!(
        "uniffi_allocation_size_{}",
        type_.canonical_name().to_snake_case()
    )
}

/// Read a value from a RustBuffer
pub fn read(type_: &Type) -> String {
    format!("uniffi_read_{}", type_.canonical_name().to_snake_case())
}

/// Write a value to a RustBuffer
pub fn write(type_: &Type) -> String {
    format!("uniffi_write_{}", type_.canonical_name().to_snake_case())
}

/// Check RustCallStatus and throw on errors
pub fn throw_if_error() -> String {
    "throw_if_error".into()
}

/// Check RustCallStatus and throw on errors using an error type to handle STATUS_ERROR
pub fn throw_if_error_with_throws_type(type_: &Type) -> String {
    match type_ {
        Type::Error(name) => format!("throw_if_error_{}", name.to_snake_case()),
        _ => panic!("throw_if_error_with_throws_type called for {type_:?}"),
    }
}
