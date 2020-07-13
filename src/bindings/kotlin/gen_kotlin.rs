/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{ CamelCase, MixedCase, ShoutySnakeCase };

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
#[template(
    syntax = "kt",
    escape = "none",
    path = "wrapper.kt",
)]
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

    pub fn type_kt(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            TypeReference::U32 => "Int".to_string(),
            TypeReference::U64 => "Long".to_string(),
            TypeReference::Float => "Float".to_string(),
            TypeReference::Double => "Double".to_string(),
            TypeReference::Bytes => "RustBuffer.ByValue".to_string(),
            // These types need conversation, and special handling for lifting/lowering.
            TypeReference::Boolean => "Boolean".to_string(),
            TypeReference::Enum(name) => class_name_kt(name)?,
            TypeReference::Record(name) => class_name_kt(name)?,
            TypeReference::Optional(t) => format!("{}?", type_kt(t)?),
            _ => panic!("[TODO: type_kt({:?})]", type_),
        })
    }

    pub fn type_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::Boolean => "Byte".to_string(),
            TypeReference::Enum(_) => "Int".to_string(),
            TypeReference::Record(_) => "RustBuffer.ByValue".to_string(),
            TypeReference::Optional(_) => "RustBuffer.ByValue".to_string(),
            TypeReference::Object(_) => "Long".to_string(),
            _ => type_kt(type_)?,
        })
    }

    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn enum_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    pub fn lower_kt(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => format!(
                "(lowerOptional({}, {{ v -> {} }}, {{ (v, buf) -> {} }})",
                nm,
                lowers_into_size_kt(&"v", type_)?,
                lower_into_kt(&"v", &"buf", type_)?
            ),
            _ => format!("{}.lower()", nm),
        })
    }

    pub fn lower_into_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => format!(
                "(lowerIntoOptional({}, {}, {{ (v, buf) -> {} }})",
                nm,
                target,
                lower_into_kt(&"v", &"buf", type_)?
            ),
            _ => format!("{}.lowerInto({})", nm, target),
        })
    }

    pub fn lowers_into_size_kt(
        nm: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(_) => format!(
                "(lowersIntoSizeOptional({}, {{ v -> {} }})",
                nm,
                lowers_into_size_kt(&"v", type_)?
            ),
            _ => format!("{}.lowersIntoSize()", nm),
        })
    }

    pub fn lift_kt(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(t) => format!(
                "liftOptional({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            _ => format!("{}.lift({})", type_kt(type_)?, nm),
        })
    }

    pub fn lift_from_kt(
        nm: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            TypeReference::Optional(t) => format!(
                "liftFromOptional({}, {{ buf -> {} }})",
                nm,
                lift_from_kt(&"buf", t)?
            ),
            _ => format!("{}.liftFrom({})", type_kt(type_)?, nm),
        })
    }
}
