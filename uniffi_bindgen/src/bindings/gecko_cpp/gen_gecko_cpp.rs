/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, SnakeCase};

use crate::interface::*;

use super::namespace_to_file_name;

// Some config options for the caller to customize the generated Gecko bindings.
// Note that this can only be used to control details *that do not affect the
// underlying component*, since the details of the underlying component are
// entirely determined by the `ComponentInterface`.
pub struct Config {
    // ...
}

impl Config {
    pub fn from(_ci: &ComponentInterface) -> Self {
        Config {
            // ...
        }
    }
}

/// A header file...
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "HeaderTemplate.h")]
pub struct Header<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> Header<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config, ci }
    }
}

/// A source file...
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "SourceTemplate.cpp")]
pub struct Source<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> Source<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config, ci }
    }
}

/// Filters for our Askama templates above. These output C++, XPIDL, and
/// WebIDL.
mod filters {
    use super::*;
    use std::fmt;

    /// Declares a C type in the `extern` declarations.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
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
            FFIType::RustCString => "const char*".into(),
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::RustError => "RustError".into(),
            FFIType::ForeignBytes => "ForeignBytes".into(),
        })
    }

    /// Declares a C++ type.
    pub fn type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "int8_t".into(),
            Type::UInt8 => "uint8_t".into(),
            Type::Int16 => "int16_t".into(),
            Type::UInt16 => "uint16_t".into(),
            Type::Int32 => "int32_t".into(),
            Type::UInt32 => "uint32_t".into(),
            Type::Int64 => "int64_t".into(),
            Type::UInt64 => "uint64_t".into(),
            Type::Float32 => "float".into(),
            Type::Float64 => "double".into(),
            Type::Boolean => "bool".into(),
            Type::String => "nsString".into(),
            Type::Enum(name) | Type::Record(name) | Type::Error(name) => class_name_cpp(name)?,
            Type::Object(name) => format!("RefPtr<{}>", class_name_cpp(name)?),
            Type::Optional(inner) => format!("Maybe<{}>", type_cpp(inner)?),
            Type::Sequence(inner) => format!("nsTArray<{}>", type_cpp(inner)?),
            Type::Map(inner) => format!("HashMap<nsString, {}>", type_cpp(inner)?),
        })
    }

    /// Declares a C++ in or out argument type.
    pub fn arg_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::String => "const nsAString&".into(),
            Type::Object(name) => format!("{}&", class_name_cpp(&name)?),
            Type::Optional(_) | Type::Record(_) | Type::Map(_) | Type::Sequence(_) => {
                format!("const {}&", type_cpp(type_)?)
            }
            _ => type_cpp(type_)?,
        })
    }

    /// Generates a dummy value for a given return type. A C++ function that
    /// declares a return type must return some value of that type, even if it
    /// throws a DOM exception via the `ErrorResult`.
    pub fn dummy_ret_value_cpp(return_type: &Type) -> Result<String, askama::Error> {
        Ok(match return_type {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => "0".into(),
            Type::Float32 => "0.0f".into(),
            Type::Float64 => "0.0".into(),
            Type::Boolean => "false".into(),
            Type::Enum(name) => format!("{}::EndGuard_", class_name_cpp(name)?),
            Type::Object(_) => "nullptr".into(),
            Type::String => "EmptyString()".into(),
            Type::Optional(_) => "Nothing()".into(),
            Type::Record(name) => format!("{} {{}}", class_name_cpp(name)?),
            Type::Map(inner) => format!("HashMap<nsString, {}>()", type_cpp(inner)?),
            Type::Sequence(inner) => format!("nsTArray<{}>()", type_cpp(inner)?),
            Type::Error(_) => panic!("[TODO: dummy_ret_value_cpp({:?})]", return_type),
        })
    }

    /// Generates an expression for lowering a C++ type into a C type when
    /// calling an FFI function.
    pub fn lower_cpp(type_: &Type, from: &str) -> Result<String, askama::Error> {
        let lifted = match type_ {
            // Since our in argument type is `nsAString`, we need to use that
            // to instantiate `ViaFfi`, not `nsString`.
            Type::String => "nsAString".into(),
            _ => type_cpp(type_)?,
        };
        Ok(format!(
            "detail::ViaFfi<{}, {}>::Lower({})",
            lifted,
            type_ffi(&FFIType::from(type_))?,
            from
        ))
    }

    /// Generates an expression for lifting a C return type from the FFI into a
    /// C++ out parameter.
    pub fn lift_cpp(type_: &Type, from: &str, into: &str) -> Result<String, askama::Error> {
        Ok(format!(
            "detail::ViaFfi<{}, {}>::Lift({}, {})",
            type_cpp(type_)?,
            type_ffi(&FFIType::from(type_))?,
            from,
            into,
        ))
    }

    pub fn var_name_webidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn enum_variant_webidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn class_name_webidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn class_name_cpp(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_webidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// For interface implementations, function and methods names are
    /// UpperCamelCase, even though they're mixedCamelCase in WebIDL.
    pub fn fn_name_cpp(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn field_name_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(format!("m{}", nm.to_camel_case()))
    }

    pub fn enum_variant_cpp(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn header_name_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(namespace_to_file_name(nm))
    }

    pub fn namespace_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_snake_case())
    }
}
