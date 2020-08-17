/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase};

use crate::interface::*;

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

    // Generates a random UUID in the lowercase hyphenated form that Gecko uses
    // for interface and component IDs (IIDs and CIDs).
    pub fn uuid(&self) -> String {
        // XXX
        "1234567".into()
    }
}

#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "HeaderTemplate.h")]
pub struct Header<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> Header<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config: config, ci }
    }
}

#[derive(Template)]
#[template(syntax = "webidl", escape = "none", path = "WebIDLTemplate.webidl")]
pub struct WebIdl<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> WebIdl<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config: config, ci }
    }
}

#[derive(Template)]
#[template(syntax = "xpidl", escape = "none", path = "XPIDLTemplate.idl")]
pub struct XpIdl<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> XpIdl<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config: config, ci }
    }
}

#[derive(Template)]
#[template(syntax = "cpp", escape = "none", path = "wrapper.cpp")]
pub struct GeckoWrapper<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> GeckoWrapper<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config: config, ci }
    }
}

/// Filters for our Askama templates above. These output C++, XPIDL, and
/// WebIDL.
mod filters {
    use super::*;
    use std::fmt;

    /// Declares an XPIDL type in the interface for this library.
    pub fn type_xpidl(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // XPIDL doesn't have a signed 8-bit integer type, so we use a
            // 16-bit integer, and rely on the reading and writing code to
            // check the range.
            Type::Int8 => "short".into(),
            Type::UInt8 => "octet".into(),
            Type::Int16 => "short".into(),
            Type::UInt16 => "unsigned short".into(),
            Type::Int32 => "long".into(),
            Type::UInt32 => "unsigned long".into(),
            Type::Int64 => "long long".into(),
            Type::UInt64 => "unsigned long long".into(),
            Type::Float32 => "float".into(),
            Type::Float64 => "double".into(),
            Type::Boolean => "boolean".into(),
            Type::String => "ACString".into(),
            Type::Enum(name) | Type::Record(name) | Type::Object(name) | Type::Error(name) => {
                panic!("uhh")
            }
            // TODO: XPIDL non-primitive arguments are all nullable by default.
            // For enums, records, objects, and errors, we'll need to add a
            // runtime "not null" check. For optionals...I guess we could use
            // a `jsval`? The problem with `jsval` is, it could be anything,
            // and we want to throw a type error if it's anything except
            // `null`. We can do that at runtime, since we know the FFI
            // signature, but it's gross.
            //
            // Another option is to scrap XPIDL generation and just make a
            // `[ChromeOnly]` WebIDL binding. Shoehorning arguments through
            // XPIDL complicates so many things.
            Type::Optional(type_) => type_xpidl(type_)?,
            Type::Sequence(type_) => format!("Array<{}>", type_xpidl(type_)?),
        })
    }

    /// Declares a WebIDL type in the interface for this library.
    pub fn type_webidl(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // ...But WebIDL does have a signed 8-bit integer type!
            Type::Int8 => "byte".into(),
            Type::UInt8 => "octet".into(),
            Type::Int16 => "short".into(),
            Type::UInt16 => "unsigned short".into(),
            Type::Int32 => "long".into(),
            Type::UInt32 => "unsigned long".into(),
            Type::Int64 => "long long".into(),
            Type::UInt64 => "unsigned long long".into(),
            Type::Float32 => "float".into(),
            // Note: Not `unrestricted double`; we don't want to allow NaNs
            // and infinity.
            Type::Float64 => "double".into(),
            Type::Boolean => "boolean".into(),
            Type::String => "DOMString".into(),
            Type::Enum(name) | Type::Record(name) | Type::Object(name) | Type::Error(name) => {
                panic!("uhh")
            }
            Type::Optional(type_) => format!("{}?", type_webidl(type_)?),
            Type::Sequence(type_) => format!("sequence<{}>", type_webidl(type_)?),
        })
    }

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
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::RustString => "char*".into(),
            FFIType::RustError => "NativeRustError".into(),
            FFIType::ForeignStringRef => "const char*".into(),
        })
    }

    /// Declares the type of an argument for the C++ binding.
    pub fn arg_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "int16_t".into(),
            Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Boolean => type_cpp(type_)?,
            Type::String => "const nsACString&".into(),
            Type::Enum(name) | Type::Record(name) => format!("{}*", name),
            Type::Object(name) => format!("{}*", interface_name_xpidl(name)?),
            Type::Error(name) => panic!("[TODO: type_cpp({:?})]", type_),
            Type::Optional(_) => panic!("[TODO: type_cpp({:?})]", type_),
            Type::Sequence(type_) => format!("const {}&", type_cpp(type_)?),
        })
    }

    fn type_cpp(type_: &Type) -> Result<String, askama::Error> {
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
            Type::String => "nsCString".into(),
            Type::Object(name) => format!("nsCOMPtr<{}>", interface_name_xpidl(name)?),
            Type::Enum(name) | Type::Record(name) => format!("RefPtr<{}>", name),
            Type::Error(name) => panic!("[TODO: type_cpp({:?})]", type_),
            Type::Optional(_) => panic!("[TODO: type_cpp({:?})]", type_),
            Type::Sequence(type_) => format!("nsTArray<{}>", type_cpp(type_)?),
        })
    }

    /// Declares the type of a return value from C++.
    pub fn ret_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "int16_t*".into(),
            Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Boolean => format!("{}*", type_cpp(type_)?),
            Type::String => "nsACString&".into(),
            Type::Object(name) => format!("getter_AddRefs<{}>", interface_name_xpidl(name)?),
            Type::Enum(name) | Type::Record(name) | Type::Error(name) => {
                panic!("[TODO: ret_type_cpp({:?})]", type_)
            }
            Type::Optional(_) => panic!("[TODO: ret_type_cpp({:?})]", type_),
            Type::Sequence(type_) => format!("{}&", type_cpp(type_)?),
        })
    }

    pub fn interface_name_xpidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(format!("mozI{}", nm.to_string().to_camel_case()))
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

    pub fn fn_name_xpidl(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    pub fn class_name_cpp(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// For interface implementations, function and methods names are
    // UpperCamelCase, even though they're mixedCamelCase in XPIDL.
    pub fn fn_name_cpp(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn lift_cpp(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let ffi_type = FFIType::from(type_);
        Ok(format!(
            "detail::ViaFfi<{}, {}>::Lift({})",
            type_cpp(type_)?,
            type_ffi(&ffi_type)?,
            name
        ))
    }

    pub fn lower_cpp(name: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let ffi_type = FFIType::from(type_);
        Ok(format!(
            "detail::ViaFfi<{}, {}>::Lower({})",
            type_cpp(type_)?,
            type_ffi(&ffi_type)?,
            name
        ))
    }
}
