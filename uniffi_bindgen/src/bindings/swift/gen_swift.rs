/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

/// Config options for the caller to customize the generated Swift.
///
/// Note that this can only be used to control details of the Swift *that do not affect the underlying component*,
/// since the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    cdylib_name: Option<String>,
    module_name: Option<String>,
    ffi_module_name: Option<String>,
    ffi_module_filename: Option<String>,
    generate_module_map: Option<bool>,
}

impl Config {
    /// The name of the Swift module containing the high-level foreign-language bindings.
    pub fn module_name(&self) -> String {
        match self.module_name.as_ref() {
            Some(name) => name.clone(),
            None => "uniffi".into(),
        }
    }

    /// The name of the lower-level C module containing the FFI declarations.
    pub fn ffi_module_name(&self) -> String {
        match self.ffi_module_name.as_ref() {
            Some(name) => name.clone(),
            None => format!("{}FFI", self.module_name()),
        }
    }

    /// The filename stem for the lower-level C module containing the FFI declarations.
    pub fn ffi_module_filename(&self) -> String {
        match self.ffi_module_filename.as_ref() {
            Some(name) => name.clone(),
            None => self.ffi_module_name(),
        }
    }

    /// The name of the `.modulemap` file for the lower-level C module with FFI declarations.
    pub fn modulemap_filename(&self) -> String {
        format!("{}.modulemap", self.ffi_module_filename())
    }

    /// The name of the `.h` file for the lower-level C module with FFI declarations.
    pub fn header_filename(&self) -> String {
        format!("{}.h", self.ffi_module_filename())
    }

    /// The name of the compiled Rust library containing the FFI implementation.
    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

    /// Whether to generate a `.modulemap` file for the lower-level C module with FFI declarations.
    pub fn generate_module_map(&self) -> bool {
        self.generate_module_map.unwrap_or(true)
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            module_name: Some(ci.namespace().into()),
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
            ..Default::default()
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            module_name: self.module_name.merge_with(&other.module_name),
            ffi_module_name: self.ffi_module_name.merge_with(&other.ffi_module_name),
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
            ffi_module_filename: self
                .ffi_module_filename
                .merge_with(&other.ffi_module_filename),
            generate_module_map: self
                .generate_module_map
                .merge_with(&other.generate_module_map),
        }
    }
}

/// Template for generating the `.h` file that defines the low-level C FFI.
///
/// This file defines only the low-level structs and functions that are exposed
/// by the compiled Rust code. It gets wrapped into a higher-level API by the
/// code from [`SwiftWrapper`].
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "BridgingHeaderTemplate.h")]
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

/// Template for generating the `.modulemap` file that exposes the low-level C FFI.
///
/// This file defines how the low-level C FFI from [`BridgingHeader`] gets exposed
/// as a Swift module that can be called by other Swift code. In our case, its only
/// job is to define the *name* of the Swift module that will contain the FFI functions
/// so that it can be imported by the higher-level code in from [`SwiftWrapper`].
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "ModuleMapTemplate.modulemap")]
pub struct ModuleMap<'config, 'ci> {
    config: &'config Config,
    _ci: &'ci ComponentInterface,
}

impl<'config, 'ci> ModuleMap<'config, 'ci> {
    pub fn new(config: &'config Config, _ci: &'ci ComponentInterface) -> Self {
        Self { config, _ci }
    }
}

/// Template for generating the `.modulemap` file that exposes the low-level C FFI.
///
/// This file wraps the low-level C FFI from [`BridgingHeader`] into a more ergonomic,
/// higher-level Swift API. It's the part that knows about API-level concepts like
/// Objects and Records and so-forth.
#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "wrapper.swift")]
pub struct SwiftWrapper<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> SwiftWrapper<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config, ci }
    }
}

/// Filters for our Askama templates above.
///
/// These output C (for the bridging header) and Swift (for the actual library) declarations,
/// mostly for dealing with types.
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
            Type::Timestamp => "Date".into(),
            Type::Duration => "TimeInterval".into(),
            Type::Enum(name)
            | Type::Record(name)
            | Type::Object(name)
            | Type::Error(name)
            | Type::CallbackInterface(name) => class_name_swift(name)?,
            Type::Optional(type_) => format!("{}?", type_swift(type_)?),
            Type::Sequence(type_) => format!("[{}]", type_swift(type_)?),
            Type::Map(type_) => format!("[String:{}]", type_swift(type_)?),
            Type::Imported(..) => todo!("not sure what to so here!"),
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
            FFIType::RustArcPtr => "void*_Nonnull".into(),
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::ForeignBytes => "ForeignBytes".into(),
            FFIType::ForeignCallback => unimplemented!("Callback interfaces are not implemented"),
        })
    }

    /// Render a literal value for Swift code.
    pub fn literal_swift(literal: &Literal) -> Result<String, askama::Error> {
        fn typed_number(type_: &Type, num_str: String) -> Result<String, askama::Error> {
            Ok(match type_ {
                // special case Int32.
                Type::Int32 => num_str,
                // otherwise use constructor e.g. UInt8(x)
                Type::Int8
                | Type::UInt8
                | Type::Int16
                | Type::UInt16
                | Type::UInt32
                | Type::Int64
                | Type::UInt64
                | Type::Float32
                | Type::Float64 => format!("{}({})", type_swift(type_)?, num_str),
                _ => panic!("Unexpected literal: {} is not a number", num_str),
            })
        }

        Ok(match literal {
            Literal::Boolean(v) => format!("{}", v),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "nil".into(),
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "[:]".into(),
            Literal::Enum(v, _) => format!(".{}", enum_variant_swift(v)?),
            Literal::Int(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("0o{:o}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            )?,
            Literal::UInt(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("0o{:o}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            )?,
            Literal::Float(string, type_) => typed_number(type_, string.clone())?,
        })
    }

    /// Lower a Swift type into an FFI type.
    ///
    /// This is used to pass arguments over the FFI, from Swift to Rust.
    pub fn lower_swift(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        match type_ {
            Type::Duration => Ok(format!(
                "{}.lower{}()",
                var_name_swift(name)?,
                type_.canonical_name()
            )),
            _ => Ok(format!("{}.lower()", var_name_swift(name)?)),
        }
    }

    /// Lift a Swift type from an FFI type.
    ///
    /// This is used to receive values over the FFI, from Rust to Swift.
    pub fn lift_swift(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        match type_ {
            Type::Duration => Ok(format!(
                "{}.lift{}({})",
                type_swift(type_)?,
                type_.canonical_name(),
                name
            )),
            _ => Ok(format!("{}.lift({})", type_swift(type_)?, name)),
        }
    }

    /// Read a Swift type from a byte buffer.
    ///
    /// This is used to receive values over the FFI, when they're part of a complex type
    /// that is passed by serializing into bytes.
    pub fn read_swift(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        match type_ {
            Type::Duration => Ok(format!(
                "{}.read{}(from: {})",
                type_swift(type_)?,
                type_.canonical_name(),
                name
            )),
            _ => Ok(format!("{}.read(from: {})", type_swift(type_)?, name)),
        }
    }

    /// Render the idiomatic Swift casing for the name of an enum.
    pub fn enum_variant_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Render the idiomatic Swift casing for the name of a class.
    pub fn class_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// Render the idiomatic Swift casing for the name of a function.
    pub fn fn_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Render the idiomatic Swift casing for the name of a variable.
    pub fn var_name_swift(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }
}
