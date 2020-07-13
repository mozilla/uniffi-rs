/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;

use anyhow::Result;
use askama::Template;

use crate::interface::*;

// Some config options for it the caller wants to customize the generated python.
// Note that this can only be used to control details of the python *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
pub struct Config {
    // No config options yet.
}

impl Config {
    pub fn from(_ci: &ComponentInterface) -> Self {
        Config {
            // No config options yet
        }
    }
}

#[derive(Template)]
#[template(
    syntax = "c",
    escape = "none",
    path = "Template-Bridging-Header.h"
)]
pub struct BridgingHeader<'config, 'ci> {
    _config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> BridgingHeader<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self {
            _config: config,
            ci,
        }
    }
}

#[derive(Template)]
#[template(
    syntax = "c",
    escape = "none",
    path = "ModuleMapTemplate.modulemap"
)]
pub struct ModuleMap<'ci, 'header> {
    ci: &'ci ComponentInterface,
    header: &'header Path,
}

impl<'ci, 'header> ModuleMap<'ci, 'header> {
    pub fn new(ci: &'ci ComponentInterface, header: &'header Path) -> Self {
        Self { ci, header }
    }
}

#[derive(Template)]
#[template(
    syntax = "swift",
    escape = "none",
    path = "wrapper.swift"
)]
pub struct SwiftWrapper<'config, 'ci> {
    _config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> SwiftWrapper<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self {
            _config: config,
            ci,
        }
    }
}

/// Filters for our Askama templates above. These output C (for the bridging
/// header) and Swift (for the actual library) declarations.
mod filters {
    use super::*;
    use std::fmt;

    /// Declares a C type in the bridging header.
    pub fn decl_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native types map nicely to the FFI without conversion.
            TypeReference::U32 => "uint32_t".into(),
            TypeReference::U64 => "uint64_t".into(),
            TypeReference::Float => "float".into(),
            TypeReference::Double => "double".into(),
            TypeReference::Bytes => "RustBuffer".into(),
            // Our FFI lowers Booleans into bytes, to work around JNA bugs.
            // We'll lift these up into Booleans on the Swift side.
            TypeReference::Boolean => "uint8_t".into(),
            // These types need conversion, and special handling for lifting/lowering.
            TypeReference::Enum(_) => "uint32_t".into(),
            TypeReference::Record(_) => "RustBuffer".into(),
            TypeReference::Optional(_) => "RustBuffer".into(),
            TypeReference::Object(_) => "uint64_t".into(),
            _ => panic!("[TODO: decl_c({:?})", type_),
        })
    }

    /// Declares a Swift type in the public interface for the library.
    pub fn decl_swift(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => "UInt32".into(),
            TypeReference::U64 => "UInt64".into(),
            TypeReference::Float => "Float".into(),
            TypeReference::Double => "Double".into(),
            // TypeReference::Bytes => "Data".into(),
            TypeReference::Boolean => "Bool".into(),
            TypeReference::Enum(name) => name.into(),
            TypeReference::Record(name) => name.into(),
            TypeReference::Optional(type_) => format!("{}?", decl_swift(type_)?),
            TypeReference::Object(name) => name.into(),
            _ => panic!("[TODO: decl_swift({:?})", type_),
        })
    }

    /// Lowers a Swift type into a C type. This is used to pass arguments over
    /// the FFI, from Swift to Rust.
    pub fn lower_swift(
        name: &dyn fmt::Display,
        _type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(format!("{}.toFFIValue()", name))
    }

    /// ...
    pub fn lift_from_swift(
        name: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(format!("{}.lift(from: {})", decl_swift(type_)?, name))
    }

    /// ...
    pub fn lift_swift(
        name: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(format!("{}.fromFFIValue({})", decl_swift(type_)?, name))
    }

    /// ...
    pub fn decl_enum_variant_swift(name: &str) -> Result<String, askama::Error> {
        use heck::MixedCase;
        Ok(name.to_mixed_case())
    }

    /// ...
    pub fn header_path(path: &Path) -> Result<String, askama::Error> {
        Ok(path.to_str().expect("Invalid bridging header path").into())
    }
}