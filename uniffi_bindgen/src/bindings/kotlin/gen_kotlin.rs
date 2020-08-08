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

    pub fn type_kt(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            // Note that unsigned integers in Kotlin are currently experimental, and we use the signed
            // variants to represent both signed and unsigned types from the component API.
            // I *think* this means you get the two's-compliment signed equivalent of unsigned values?
            // That's probably going to get messy, but not sure there's a better way...
            Type::Int8 | Type::UInt8 => "Byte".to_string(),
            Type::Int16 | Type::UInt16 => "Short".to_string(),
            Type::Int32 | Type::UInt32 => "Int".to_string(),
            Type::Int64 | Type::UInt64 => "Long".to_string(),
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
        })
    }

    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            // Note that unsigned integers in Kotlin are currently experimental, and we use the signed
            // variants to represent both signed and unsigned types from the component API.
            // I *think* this means you get the two's-compliment signed equivalent of unsigned values?
            // That's probably going to get messy, but not sure there's a better way...
            FFIType::Int8 | FFIType::UInt8 => "Byte".to_string(),
            FFIType::Int16 | FFIType::UInt16 => "Short".to_string(),
            FFIType::Int32 | FFIType::UInt32 => "Int".to_string(),
            FFIType::Int64 | FFIType::UInt64 => "Long".to_string(),
            FFIType::Float32 => "Float".to_string(),
            FFIType::Float64 => "Double".to_string(),
            FFIType::RustBuffer => "RustBuffer.ByValue".to_string(),
            FFIType::RustString => "Pointer".to_string(),
            FFIType::RustError => "RustError".to_string(),
            // Kotlin+JNA has some magic to pass its native string type as char* pointers.
            FFIType::ForeignStringRef => "String".to_string(),
        })
    }

    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn var_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn enum_variant_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    pub fn lower_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::Optional(t) => format!(
                "lowerOptional({}, {{ v -> {} }}, {{ v, buf -> {} }})",
                nm,
                lowers_into_size_kt(&"v", t)?,
                lower_into_kt(&"v", &"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "lowerSequence({}, {{ v -> {} }}, {{ v, buf -> {} }})",
                nm,
                lowers_into_size_kt(&"v", t)?,
                lower_into_kt(&"v", &"buf", t)?
            ),
            _ => format!("{}.lower()", nm),
        })
    }

    pub fn lower_into_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::Optional(t) => format!(
                "lowerIntoOptional({}, {}, {{ v, buf -> {} }})",
                nm,
                target,
                lower_into_kt(&"v", &"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "lowerIntoSequence({}, {}, {{ v, buf -> {} }})",
                nm,
                target,
                lower_into_kt(&"v", &"buf", t)?
            ),
            _ => format!("{}.lowerInto({})", nm, target),
        })
    }

    pub fn lowers_into_size_kt(
        nm: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::Optional(t) => format!(
                "lowersIntoSizeOptional({}, {{ v -> {} }})",
                nm,
                lowers_into_size_kt(&"v", t)?
            ),
            Type::Sequence(t) => format!(
                "lowersIntoSizeSequence({}, {{ v -> {} }})",
                nm,
                lowers_into_size_kt(&"v", t)?
            ),
            _ => format!("{}.lowersIntoSize()", nm),
        })
    }

    pub fn lift_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::Optional(t) => format!(
                "liftOptional({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "liftSequence({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            _ => format!("{}.lift({})", type_kt(type_)?, nm),
        })
    }

    pub fn lift_from_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::Optional(t) => format!(
                "liftFromOptional({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            Type::Sequence(t) => format!(
                "liftFromSequence({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            _ => format!("{}.liftFrom({})", type_kt(type_)?, nm),
        })
    }
}
