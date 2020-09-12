/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};

use crate::interface::*;

// Some config options for it the caller wants to customize the generated Kotlin.
// Note that this can only be used to control details of the Kotlin *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
pub struct Config {
    pub package_name: String,
}

impl Config {
    pub fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: format!("uniffi.{}", ci.namespace()),
        }
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "wrapper.kt")]
pub struct KotlinWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    /// Get the Kotlin syntax for representing a given api-level `Type`.
    pub fn type_kt(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            Type::UInt8 => "UByte".to_string(),
            Type::UInt16 => "UShort".to_string(),
            Type::UInt32 => "UInt".to_string(),
            Type::UInt64 => "ULong".to_string(),
            Type::Int8 => "Byte".to_string(),
            Type::Int16 => "Short".to_string(),
            Type::Int32 => "Int".to_string(),
            Type::Int64 => "Long".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Double".to_string(),
            // These types need conversion, and special handling for lifting/lowering.
            Type::Boolean => "Boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Enum(name) | Type::Record(name) | Type::Object(name) | Type::Error(name) => {
                class_name_kt(name)?
            }
            Type::Optional(t) => format!("{}?", type_kt(t)?),
            Type::Sequence(t) => format!("List<{}>", type_kt(t)?),
            Type::Map(t) => format!("Map<String, {}>", type_kt(t)?),
        })
    }

    /// Get the Kotlin syntax for representing a given low-level `FFIType`.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            // Note that unsigned integers in Kotlin are currently experimental, but java.nio.ByteBuffer does not
            // support them yet. Thus, we use the signed variants to represent both signed and unsigned
            // types from the component API.
            FFIType::Int8 | FFIType::UInt8 => "Byte".to_string(),
            FFIType::Int16 | FFIType::UInt16 => "Short".to_string(),
            FFIType::Int32 | FFIType::UInt32 => "Int".to_string(),
            FFIType::Int64 | FFIType::UInt64 => "Long".to_string(),
            FFIType::Float32 => "Float".to_string(),
            FFIType::Float64 => "Double".to_string(),
            FFIType::RustCString => "Pointer".to_string(),
            FFIType::RustBuffer => "RustBuffer.ByValue".to_string(),
            FFIType::RustError => "RustError".to_string(),
            FFIType::ForeignBytes => "ForeignBytes.ByValue".to_string(),
        })
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    /// Get a Kotlin expression for lowering a value into something we can pass over the FFI.
    ///
    /// Where possible, this delegates to a `lower()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn lower_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::Optional(t) => format!(
                "lowerOptional({}, {{ v, buf -> {} }})",
                nm,
                write_kt(&"v", &"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "lowerSequence({}, {{ v, buf -> {} }})",
                nm,
                write_kt(&"v", &"buf", t)?
            ),
            Type::Map(t) => format!(
                "lowerMap({}, {{ k, v, buf -> {}; {} }})",
                nm,
                write_kt(&"k", &"buf", &Type::String)?,
                write_kt(&"v", &"buf", t)?
            ),
            _ => format!("{}.lower()", nm),
        })
    }

    /// Get a Kotlin expression for writing a value into a byte buffer.
    ///
    /// Where possible, this delegates to a `write()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn write_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::Optional(t) => format!(
                "writeOptional({}, {}, {{ v, buf -> {} }})",
                nm,
                target,
                write_kt(&"v", &"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "writeSequence({}, {}, {{ v, buf -> {} }})",
                nm,
                target,
                write_kt(&"v", &"buf", t)?
            ),
            Type::Map(t) => format!(
                "writeMap({}, {}, {{ k, v, buf -> {}; {} }})",
                nm,
                target,
                write_kt(&"k", &"buf", &Type::String)?,
                write_kt(&"v", &"buf", t)?
            ),
            _ => format!("{}.write({})", nm, target),
        })
    }

    /// Get a Kotlin expression for lifting a value from something we received over the FFI.
    ///
    /// Where possible, this delegates to a `lift()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn lift_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::Optional(t) => {
                format!("liftOptional({}, {{ buf -> {} }})", nm, read_kt(&"buf", t)?)
            }
            Type::Sequence(t) => {
                format!("liftSequence({}, {{ buf -> {} }})", nm, read_kt(&"buf", t)?)
            }
            Type::Map(t) => format!(
                "liftMap({}, {{ buf -> Pair({}, {}) }})",
                nm,
                read_kt(&"buf", &Type::String)?,
                read_kt(&"buf", t)?
            ),
            _ => format!("{}.lift({})", type_kt(type_)?, nm),
        })
    }

    /// Get a Kotlin expression for reading a value from a byte buffer.
    ///
    /// Where possible, this delegates to a `read()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn read_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::Optional(t) => {
                format!("readOptional({}, {{ buf -> {} }})", nm, read_kt(&"buf", t)?)
            }
            Type::Sequence(t) => {
                format!("readSequence({}, {{ buf -> {} }})", nm, read_kt(&"buf", t)?)
            }
            Type::Map(t) => format!(
                "readMap({}, {{ buf -> Pair({}, {}) }})",
                nm,
                read_kt(&"buf", &Type::String)?,
                read_kt(&"buf", t)?
            ),
            _ => format!("{}.read({})", type_kt(type_)?, nm),
        })
    }
}
