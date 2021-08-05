/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, SnakeCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

// via https://doc.rust-lang.org/reference/keywords.html
#[rustfmt::skip]
const RESERVED_WORDS: &[&str] = &[
    // strict keywords
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while",

    // reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",

    // weak keywords
    "union",
];

fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

// Some config options for it the caller wants to customize the generated rust.
// Note that this can only be used to control details of the rust *that do not affect the underlying component*,
// since the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config;

impl From<&ComponentInterface> for Config {
    fn from(_ci: &ComponentInterface) -> Self {
        Config
    }
}

impl MergeWith for Config {
    fn merge_with(&self, _other: &Self) -> Self {
        Config
    }
}

#[derive(Template)]
#[template(syntax = "rs", escape = "none", path = "wrapper.rs")]
pub struct RustWrapper<'a> {
    ci: &'a ComponentInterface,
}
impl<'a> RustWrapper<'a> {
    pub fn new(_config: Config, ci: &'a ComponentInterface) -> Self {
        Self { ci }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    /// Get the Rust syntax for representing a given api-level `Type`.
    pub fn type_rs(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            Type::UInt8 => "u8".to_string(),
            Type::UInt16 => "u16".to_string(),
            Type::UInt32 => "u32".to_string(),
            Type::UInt64 => "u64".to_string(),
            Type::Int8 => "i8".to_string(),
            Type::Int16 => "i16".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            // These types need conversion, and special handling for lifting/lowering.
            Type::Boolean => "bool".to_string(),
            Type::String => "String".to_string(),
            Type::Timestamp => "std::time::Instant".to_string(),
            Type::Duration => "std::time::Duration".to_string(),
            Type::Enum(name)
            | Type::Record(name)
            | Type::Object(name)
            | Type::Error(name)
            | Type::CallbackInterface(name) => class_name_rs(name)?,
            Type::Optional(t) => format!("Option<{}>", type_rs(t)?),
            Type::Sequence(t) => format!("Vec<{}>", type_rs(t)?),
            Type::Map(t) => format!("std::collections::HashMap<String, {}>", type_rs(t)?),
            Type::External { .. } => panic!("no support for external types yet"),
            Type::Wrapped { .. } => panic!("no support for wrapped types yet"),
        })
    }

    pub fn mod_name_rs(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn class_name_rs(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_rs(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn var_name_rs(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        let prefix = if is_reserved_word(&nm) { "_" } else { "" };

        Ok(format!("{}{}", prefix, nm.to_snake_case()))
    }

    pub fn enum_name_rs(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }
}
