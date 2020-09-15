/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

// Some config options for it the caller wants to customize the generated python.
// Note that this can only be used to control details of the python *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    // No config options yet.
}

impl From<&ComponentInterface> for Config {
    fn from(_ci: &ComponentInterface) -> Self {
        Config {}
    }
}

impl MergeWith for Config {
    fn merge_with(&self, _other: &Self) -> Self {
        self.clone()
    }
}

#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "Template-Bridging-Header.h")]
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
#[template(syntax = "c", escape = "none", path = "ModuleMapTemplate.modulemap")]
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
#[template(syntax = "swift", escape = "none", path = "wrapper.swift")]
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

    /// Declares a Swift type in the public interface for the library.
    pub fn type_swift(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "Int8".into(),
            Type::UInt8 => "UInt8".into(),
            Type::Int16 => "Int16".into(),
            Type::UInt16 => "UInt16".into(),
            Type::Int32 => "Int32".into(),
            Type::UInt32 => "UInt32".into(),
            Type::Int64 => "Int64".into(),
            Type::UInt64 => "UInt64".into(),
            Type::Float32 => "Float".into(),
            Type::Float64 => "Double".into(),
            Type::Boolean => "Bool".into(),
            Type::String => "String".into(),
            Type::Enum(name) | Type::Record(name) | Type::Object(name) | Type::Error(name) => {
                class_name_swift(name)?
            }
            Type::Optional(type_) => format!("{}?", type_swift(type_)?),
            Type::Sequence(type_) => format!("[{}]", type_swift(type_)?),
            Type::Map(type_) => format!("[String:{}]", type_swift(type_)?),
        })
    }

    /// Declares a C type in the bridging header.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native types map nicely to the FFI without conversion.
            FFIType::Int8 => "int8_t".into(),
            FFIType::UInt8 => "uint8_t".into(),
            FFIType::Int16 => "int16_t".into(),
            FFIType::UInt16 => "uint16_t".into(),
            FFIType::Int32 => "int32_t".into(),
            FFIType::UInt32 => "uint32_t".into(),
            FFIType::Int64 => "int64_t".into(),
            FFIType::UInt64 => "uint64_t".into(),
            FFIType::Float32 => "float".into(),
            FFIType::Float64 => "double".into(),
            FFIType::RustCString => "const char*_Nonnull".into(),
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::RustError => "NativeRustError".into(),
            FFIType::ForeignBytes => "ForeignBytes".into(),
        })
    }

    pub fn literal_swift(literal: &Literal) -> Result<String, askama::Error> {
        fn typed_number(type_: &Type, num_str: String) -> Result<String, askama::Error> {
            Ok(match type_ {
                Type::Int8 => format!("Int8({})", num_str),
                Type::UInt8 => format!("UInt8({})", num_str),
                Type::Int16 => format!("Int16({})", num_str),
                Type::UInt16 => format!("UInt16({})", num_str),
                Type::Int32 => num_str,
                Type::UInt32 => format!("UInt32({})", num_str),
                Type::Int64 => format!("Int64({})", num_str),
                Type::UInt64 => format!("UInt64({})", num_str),
                Type::Float32 => format!("Float({})", num_str),
                Type::Float64 => format!("Double({})", num_str),
                _ => panic!("Unexpected literal: {} is not a number", num_str),
            })
        }

        let output = match literal {
            Literal::Boolean(v) => format!("{}", v),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "nil".into(),
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "[:]".into(),
            Literal::Int(i, radix, type_) => 
                typed_number(type_, match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i), 
                })?,
            Literal::UInt(i, radix, type_) => 
                typed_number(type_, match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i), 
                })?,
            Literal::Float(n, type_) =>
                typed_number(type_, format!("{}", n))?,
        };

        Ok(output)
    }

    /// Lower a Swift type into an FFI type.
    ///
    /// This is used to pass arguments over the FFI, from Swift to Rust.
    pub fn lower_swift(name: &dyn fmt::Display, _type_: &Type) -> Result<String, askama::Error> {
        Ok(format!("{}.lower()", var_name_swift(name)?))
    }

    /// Lift a Swift type from an FFI type.
    ///
    /// This is used to receive values over the FFI, from Rust to Swift.
    pub fn lift_swift(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        Ok(format!("{}.lift({})", type_swift(type_)?, name))
    }

    /// Read a Swift type from a byte buffer.
    ///
    /// This is used to receive values over the FFI, when they're part of a complex type
    /// that is passed by serializing into bytes.
    pub fn read_swift(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        Ok(format!("{}.read(from: {})", type_swift(type_)?, name))
    }

    pub fn enum_variant_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn class_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn var_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn header_path(path: &Path) -> Result<String, askama::Error> {
        Ok(path.to_str().expect("Invalid bridging header path").into())
    }
}
