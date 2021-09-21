/* This Source Code Form is subject to the terms of the Mozilla Publie
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::interface;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};

/// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
pub(super) fn class_name(name: &str) -> String {
    name.to_camel_case()
}

/// Get the idiomatic Kotlin rendering of a function name.
pub(super) fn fn_name(name: &str) -> String {
    name.to_mixed_case()
}

/// Get the idiomatic Kotlin rendering of a variable name.
pub(super) fn var_name(name: &str) -> String {
    name.to_mixed_case()
}

/// Get the idiomatic Kotlin rendering of an individual enum variant.
pub(super) fn enum_variant_name(name: &str) -> String {
    name.to_shouty_snake_case()
}

/// Get the idiomatic Kotlin rendering of an exception name
///
/// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
/// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
/// and is distinguished from an "Exception".
pub(super) fn error_name(name: &str) -> String {
    match name.strip_suffix("Error") {
        None => name.to_string(),
        Some(stripped) => {
            let mut kt_exc_name = stripped.to_owned();
            kt_exc_name.push_str("Exception");
            kt_exc_name
        }
    }
}

/// Get the Kotlin name for various parts of the interface.
pub(super) trait KotlinCodeName {
    /// Name for this type in Kotlin code
    fn nm(&self) -> String;
}

impl KotlinCodeName for interface::Method {
    fn nm(&self) -> String {
        fn_name(self.name())
    }
}

impl KotlinCodeName for interface::Constructor {
    fn nm(&self) -> String {
        fn_name(self.name())
    }
}

impl KotlinCodeName for interface::Field {
    fn nm(&self) -> String {
        var_name(self.name())
    }
}

impl KotlinCodeName for interface::Argument {
    fn nm(&self) -> String {
        var_name(self.name())
    }
}

impl KotlinCodeName for interface::FFIType {
    fn nm(&self) -> String {
        match self {
            // Note that unsigned integers in Kotlin are currently experimental, but java.nio.ByteBuffer does not
            // support them yet. Thus, we use the signed variants to represent both signed and unsigned
            // types from the component API.
            interface::FFIType::Int8 | interface::FFIType::UInt8 => "Byte".to_string(),
            interface::FFIType::Int16 | interface::FFIType::UInt16 => "Short".to_string(),
            interface::FFIType::Int32 | interface::FFIType::UInt32 => "Int".to_string(),
            interface::FFIType::Int64 | interface::FFIType::UInt64 => "Long".to_string(),
            interface::FFIType::Float32 => "Float".to_string(),
            interface::FFIType::Float64 => "Double".to_string(),
            interface::FFIType::RustArcPtr => "Pointer".to_string(),
            interface::FFIType::RustBuffer => "RustBuffer.ByValue".to_string(),
            interface::FFIType::ForeignBytes => "ForeignBytes.ByValue".to_string(),
            interface::FFIType::ForeignCallback => "ForeignCallback".to_string(),
        }
    }
}

/// Get the Kotlin name for enum/error variants.
pub(super) trait KotlinVariantName {
    fn variant_name(&self, variant: &interface::Variant) -> String;
}

impl KotlinVariantName for interface::Error {
    fn variant_name(&self, variant: &interface::Variant) -> String {
        error_name(variant.name())
    }
}

impl KotlinVariantName for interface::Enum {
    fn variant_name(&self, variant: &interface::Variant) -> String {
        if self.is_flat() {
            enum_variant_name(variant.name())
        } else {
            class_name(variant.name())
        }
    }
}
