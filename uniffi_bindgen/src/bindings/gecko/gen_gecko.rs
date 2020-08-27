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
}

/// Indicates whether a WebIDL type is reflected as an out parameter or return
/// value in C++. This is used by the namespace and interface templates to
/// generate the correct argument lists for the binding.
pub enum ReturnPosition<'a> {
    OutParam(&'a Type),
    Return(&'a Type),
}

impl<'a> ReturnPosition<'a> {
    /// Given a type, returns a tag indicating whether it's returned by value or
    /// via an out parameter.
    pub fn for_type(type_: &'a Type) -> ReturnPosition<'a> {
        match type_ {
            Type::String => ReturnPosition::OutParam(type_),
            Type::Optional(_) => ReturnPosition::OutParam(type_),
            Type::Record(_) => ReturnPosition::OutParam(type_),
            Type::Map(_) => ReturnPosition::OutParam(type_),
            Type::Sequence(_) => ReturnPosition::OutParam(type_),
            _ => ReturnPosition::Return(type_),
        }
    }
}

/// Returns a dummy empty value for the given type. This is used to
/// implement the `cpp::bail` macro to return early if a WebIDL function or
/// method throws an exception.
pub fn default_return_value_cpp(type_: &Type) -> Option<String> {
    Some(match type_ {
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
        Type::Enum(name) => format!("{}::EndGuard_", name),
        Type::Object(_) => "nullptr".into(),
        Type::String | Type::Record(_) | Type::Optional(_) | Type::Sequence(_) | Type::Map(_) => {
            return None
        }
        Type::Error(name) => panic!("[TODO: ret_type_cpp({:?})]", type_),
    })
}

/// A template for a Firefox WebIDL file. We only generate one of these per
/// component.
#[derive(Template)]
#[template(syntax = "webidl", escape = "none", path = "WebIDLTemplate.webidl")]
pub struct WebIdl<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> WebIdl<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config, ci }
    }
}

/// A shared header file that's included by all our bindings. This defines
/// common serialization logic and `extern` declarations for the FFI. Note that
/// the bindings always include this header file, never the other way around.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "SharedHeaderTemplate.h")]
pub struct SharedHeader<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> SharedHeader<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self { config, ci }
    }
}

/// A header file generated for a namespace with top-level functions.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "NamespaceHeaderTemplate.h")]
pub struct NamespaceHeader<'config, 'ci> {
    config: &'config Config,
    namespace: &'ci str,
    functions: &'ci [Function],
}

impl<'config, 'ci> NamespaceHeader<'config, 'ci> {
    pub fn new(config: &'config Config, namespace: &'ci str, functions: &'ci [Function]) -> Self {
        Self {
            config,
            namespace,
            functions,
        }
    }
}

/// An implementation file generated for a namespace with top-level functions.
#[derive(Template)]
#[template(syntax = "cpp", escape = "none", path = "NamespaceTemplate.cpp")]
pub struct Namespace<'config, 'ci> {
    config: &'config Config,
    namespace: &'ci str,
    functions: &'ci [Function],
}

impl<'config, 'ci> Namespace<'config, 'ci> {
    pub fn new(config: &'config Config, namespace: &'ci str, functions: &'ci [Function]) -> Self {
        Self {
            config,
            namespace,
            functions,
        }
    }
}

/// A header file generated for an interface.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "InterfaceHeaderTemplate.h")]
pub struct InterfaceHeader<'config, 'ci> {
    config: &'config Config,
    namespace: &'ci str,
    obj: &'ci Object,
}

impl<'config, 'ci> InterfaceHeader<'config, 'ci> {
    pub fn new(config: &'config Config, namespace: &'ci str, obj: &'ci Object) -> Self {
        Self {
            config,
            namespace,
            obj,
        }
    }
}

/// An implementation file generated for a namespace with top-level functions.
#[derive(Template)]
#[template(syntax = "cpp", escape = "none", path = "InterfaceTemplate.cpp")]
pub struct Interface<'config, 'ci> {
    config: &'config Config,
    namespace: &'ci str,
    obj: &'ci Object,
}

impl<'config, 'ci> Interface<'config, 'ci> {
    pub fn new(config: &'config Config, namespace: &'ci str, obj: &'ci Object) -> Self {
        Self {
            config,
            namespace,
            obj,
        }
    }
}

/// Filters for our Askama templates above. These output C++, XPIDL, and
/// WebIDL.
mod filters {
    use super::*;
    use std::fmt;

    /// Declares a WebIDL type in the interface for this library.
    pub fn type_webidl(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
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
            Type::Enum(name) | Type::Record(name) | Type::Object(name) => class_name_webidl(name)?,
            Type::Error(name) => panic!("[TODO: type_webidl({:?})]", type_),
            Type::Optional(type_) => format!("{}?", type_webidl(type_)?),
            Type::Sequence(type_) => format!("sequence<{}>", type_webidl(type_)?),
            Type::Map(type_) => format!("record<DOMString, {}>", type_webidl(type_)?),
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
            FFIType::RustError => "RustError".into(),
            FFIType::ForeignStringRef => "const char*".into(),
        })
    }

    /// Declares the type of an argument for the C++ binding.
    pub fn arg_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Boolean => type_cpp(type_)?,
            Type::String => "const nsAString&".into(),
            Type::Enum(name) => name.into(),
            Type::Record(name) | Type::Object(name) => format!("const {}&", class_name_cpp(name)?),
            Type::Error(name) => panic!("[TODO: type_cpp({:?})]", type_),
            // Nullable objects might be passed as pointers, not sure?
            Type::Optional(type_) => format!("const Nullable<{}>&", type_cpp(type_)?),
            Type::Sequence(type_) => format!("const Sequence<{}>&", type_cpp(type_)?),
            Type::Map(type_) => format!("const Record<nsString, {}>&", type_cpp(type_)?),
        })
    }

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
            Type::String => "nsAString".into(),
            Type::Enum(name) | Type::Record(name) => class_name_cpp(name)?,
            Type::Object(name) => format!("RefPtr<{}>", class_name_cpp(name)?),
            Type::Error(name) => panic!("[TODO: type_cpp({:?})]", type_),
            Type::Optional(type_) => format!("Nullable<{}>", type_cpp(type_)?),
            Type::Sequence(type_) => format!("nsTArray<{}>", type_cpp(type_)?),
            Type::Map(type_) => format!("Record<nsString, {}>", type_cpp(type_)?),
        })
    }

    /// Declares the type of a return value from C++.
    pub fn ret_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Boolean
            | Type::Enum(_) => type_cpp(type_)?,
            Type::String => "nsAString&".into(),
            Type::Object(name) => format!("already_AddRefed<{}>", class_name_cpp(name)?),
            Type::Error(name) => panic!("[TODO: ret_type_cpp({:?})]", type_),
            Type::Record(_) | Type::Optional(_) | Type::Sequence(_) | Type::Map(_) => {
                format!("{}&", type_cpp(type_)?)
            }
        })
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
        // TODO: Make sure this does the right thing for hyphenated variants.
        // Example: "bookmark-added" becomes `Bookmark_added`.
        Ok(nm.to_string().to_camel_case())
    }
}
