/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

use super::namespace_to_file_name;
use super::webidl::{
    BindingArgument, BindingFunction, ReturnBy, ReturningBindingFunction, ThrowBy,
};

/// Config options for the generated Firefox front-end bindings. Note that this
/// can only be used to control details *that do not affect the underlying
/// component*, since the details of the underlying component are entirely
/// determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    // ...
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
/// common serialization logic and `extern` declarations for the FFI. Thes
/// namespace and interface source files `#include` this file.
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

/// A header file generated for a namespace containing top-level functions. If
/// the namespace in the UniFFI IDL file is empty, this file isn't generated.
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

/// An implementation file for a namespace with top-level functions. If the
/// namespace in the UniFFI IDL is empty, this isn't generated.
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

/// A header file generated for each interface in the UniFFI IDL.
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

/// An implementation file generated for each interface in the UniFFI IDL.
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

/// Filters for our Askama templates above. These output C++ and WebIDL.
mod filters {
    use super::*;
    use std::fmt;

    /// Declares a WebIDL type.
    ///
    /// Terminology clarification: UniFFI IDL, the `ComponentInterface`,
    /// and Firefox's WebIDL use different but overlapping names for
    /// the same types.
    ///
    /// * `Type::Record` is called a "dictionary" in Firefox WebIDL. It's
    ///   represented as `dictionary T` in UniFFI IDL and WebIDL.
    /// * `Type::Object` is called an "interface" in Firefox WebIDL. It's
    ///   represented as `interface T` in UniFFI IDL and WebIDL.
    /// * `Type::Optional` is called "nullable" in Firefox WebIDL. It's
    ///   represented as `T?` in UniFFI IDL and WebIDL.
    /// * `Type::Map` is called a "record" in Firefox WebIDL. It's represented
    ///   as `record<string, T>` in UniFFI IDL, and `record<DOMString, T>` in
    ///   WebIDL.
    ///
    /// There are also semantic differences:
    ///
    /// * In UniFFI IDL, all `dictionary` members are required by default; in
    ///   WebIDL, they're all optional. The generated WebIDL file adds a
    ///   `required` keyword to each member.
    /// * In UniFFI IDL, an argument can specify a default value directly.
    ///   In WebIDL, arguments with default values must have the `optional`
    ///   keyword.
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
            Type::Optional(inner) => format!("{}?", type_webidl(inner)?),
            Type::Sequence(inner) => format!("sequence<{}>", type_webidl(inner)?),
            Type::Map(inner) => format!("record<DOMString, {}>", type_webidl(inner)?),
        })
    }

    /// Emits a literal default value for WebIDL.
    pub fn literal_webidl(literal: &Literal) -> Result<String, askama::Error> {
        Ok(match literal {
            Literal::Boolean(v) => format!("{}", v),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "null".into(),
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "{}".into(),
            Literal::Enum(v, _) => format!("\"{}\"", enum_variant_webidl(v)?),
            Literal::Int(i, radix, _) => match radix {
                Radix::Octal => format!("0{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
            Literal::UInt(i, radix, _) => match radix {
                Radix::Octal => format!("0{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
            Literal::Float(string, _) => string.into(),
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
            Type::Enum(name) | Type::Record(name) => class_name_cpp(name)?,
            Type::Object(name) => format!("OwningNonNull<{}>", class_name_cpp(name)?),
            Type::Optional(inner) => {
                // Nullable objects become `RefPtr<T>` (instead of
                // `OwningNonNull<T>`); all others become `Nullable<T>`.
                match inner.as_ref() {
                    Type::Object(name) => format!("RefPtr<{}>", class_name_cpp(name)?),
                    Type::String => "nsString".into(),
                    _ => format!("Nullable<{}>", type_cpp(inner)?),
                }
            }
            Type::Sequence(inner) => format!("nsTArray<{}>", type_cpp(inner)?),
            Type::Map(inner) => format!("Record<nsString, {}>", type_cpp(inner)?),
            Type::Error(name) => panic!("[TODO: type_cpp({:?})]", type_),
        })
    }

    fn in_arg_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Optional(inner) => match inner.as_ref() {
                Type::Object(_) | Type::String => type_cpp(type_)?,
                _ => format!("Nullable<{}>", in_arg_type_cpp(inner)?),
            },
            Type::Sequence(inner) => format!("Sequence<{}>", in_arg_type_cpp(&inner)?),
            _ => type_cpp(type_)?,
        })
    }

    /// Declares a C++ in or out argument type.
    pub fn arg_type_cpp(arg: &BindingArgument<'_>) -> Result<String, askama::Error> {
        Ok(match arg {
            BindingArgument::GlobalObject => "GlobalObject&".into(),
            BindingArgument::ErrorResult => "ErrorResult&".into(),
            BindingArgument::In(arg) => {
                // In arguments are usually passed by `const` reference for
                // object types, and by value for primitives. As an exception,
                // `nsString` becomes `nsAString` when passed as an argument,
                // and nullable objects are passed as pointers. Sequences map
                // to the `Sequence` type, not `nsTArray`.
                match arg.type_() {
                    Type::String => "const nsAString&".into(),
                    Type::Object(name) => format!("{}&", class_name_cpp(&name)?),
                    Type::Optional(inner) => match inner.as_ref() {
                        Type::String => "const nsAString&".into(),
                        Type::Object(name) => format!("{}*", class_name_cpp(&name)?),
                        _ => format!("const {}&", in_arg_type_cpp(&arg.type_())?),
                    },
                    Type::Record(_) | Type::Map(_) | Type::Sequence(_) => {
                        format!("const {}&", in_arg_type_cpp(&arg.type_())?)
                    }
                    _ => in_arg_type_cpp(&arg.type_())?,
                }
            }
            BindingArgument::Out(type_) => {
                // Out arguments are usually passed by reference. `nsString`
                // becomes `nsAString`.
                match type_ {
                    Type::String => "nsAString&".into(),
                    Type::Optional(inner) => match inner.as_ref() {
                        Type::String => "nsAString&".into(),
                        _ => format!("{}&", type_cpp(type_)?),
                    },
                    _ => format!("{}&", type_cpp(type_)?),
                }
            }
        })
    }

    /// Declares a C++ return type.
    pub fn ret_type_cpp(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Object(name) => format!("already_AddRefed<{}>", class_name_cpp(name)?),
            Type::Optional(inner) => match inner.as_ref() {
                Type::Object(name) => format!("already_AddRefed<{}>", class_name_cpp(name)?),
                _ => type_cpp(type_)?,
            },
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
            Type::Enum(name) => format!("{}::EndGuard_", name),
            Type::Object(_) => "nullptr".into(),
            Type::String => "EmptyString()".into(),
            Type::Optional(_) | Type::Record(_) | Type::Map(_) | Type::Sequence(_) => {
                format!("{}()", type_cpp(return_type)?)
            }
            Type::Error(_) => panic!("[TODO: dummy_ret_value_cpp({:?})]", return_type),
        })
    }

    pub fn detail_cpp(namespace: &str) -> Result<String, askama::Error> {
        Ok(format!("{}_detail", namespace))
    }

    /// Generates an expression for lowering a C++ type into a C type when
    /// calling an FFI function.
    pub fn lower_cpp(namespace: &str, type_: &Type, from: &str) -> Result<String, askama::Error> {
        let lifted = match type_ {
            // Since our in argument type is `nsAString`, we need to use that
            // to instantiate `ViaFfi`, not `nsString`.
            Type::String => "nsAString".into(),
            Type::Optional(inner) => match inner.as_ref() {
                Type::String => "nsAString".into(),
                _ => in_arg_type_cpp(type_)?,
            },
            _ => in_arg_type_cpp(type_)?,
        };
        let detail = detail_cpp(namespace)?;
        Ok(format!(
            "{}::ViaFfi<{}, {}>::Lower({})",
            detail,
            lifted,
            type_ffi(&FFIType::from(type_))?,
            from
        ))
    }

    /// Generates an expression for lifting a C return type from the FFI into a
    /// C++ out parameter.
    pub fn lift_cpp(
        namespace: &str,
        type_: &Type,
        from: &str,
        into: &str,
    ) -> Result<String, askama::Error> {
        let lifted = match type_ {
            // Out arguments are also `nsAString`, so we need to use it for the
            // instantiation.
            Type::String => "nsAString".into(),
            Type::Optional(inner) => match inner.as_ref() {
                Type::String => "nsAString".into(),
                _ => type_cpp(type_)?,
            },
            _ => type_cpp(type_)?,
        };
        let detail = detail_cpp(namespace)?;
        Ok(format!(
            "{}::ViaFfi<{}, {}>::Lift({}, {})",
            detail,
            lifted,
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
        // TODO: Make sure this does the right thing for hyphenated variants.
        // Example: "bookmark-added" should become `Bookmark_added`, because
        // that's what Firefox's `Codegen.py` spits out.
        //
        // https://github.com/mozilla/uniffi-rs/issues/294
        Ok(nm.to_string().to_camel_case())
    }

    pub fn header_name_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(namespace_to_file_name(nm))
    }
}
