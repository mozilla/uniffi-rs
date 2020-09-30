/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

use super::webidl::{
    BindingArgument, BindingFunction, ReturnBy, ReturningBindingFunction, ThrowBy,
};

/// Config options for the generated Firefox front-end bindings. Note that this
/// can only be used to control details *that do not affect the underlying
/// component*, since the details of the underlying component are entirely
/// determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Specifies an optional prefix to use for all definitions (interfaces,
    /// dictionaries, enums, and namespaces) in the generated Firefox WebIDL
    /// binding. If a prefix is not specified, the Firefox WebIDL definitions
    /// will use the same names as the UDL.
    ///
    /// For example, if the prefix is `Hola`, and the UDL for the component
    /// declares `namespace foo`, `dictionary Bar`, and `interface Baz`, the
    /// definitions will be exposed in Firefox WebIDL as `HolaFoo`, `HolaBar`,
    /// and `HolaBaz`.
    ///
    /// This option exists because definition names all share a global
    /// namespace (further, all WebIDL namespaces, interfaces, and enums are
    /// exposed on `window`), so they must be unique. Firefox will fail to
    /// compile if two different WebIDL files declare interfaces, dictionaries,
    /// enums, or namespaces with the same name.
    ///
    /// For this reason, web standards often prefix their definitions: for
    /// example, the dictionary to create a `PushSubscription` is called
    /// `PushSubscriptionOptionsInit`, not just `Init`. For UniFFI components,
    /// prefixing definitions in UDL would make it awkward to consume from other
    /// languages that _do_ have namespaces.
    ///
    /// So we expose the prefix as an option for just Gecko JS bindings.
    pub definition_prefix: Option<String>,
}

impl From<&ComponentInterface> for Config {
    fn from(_ci: &ComponentInterface) -> Self {
        Config::default()
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            definition_prefix: self.definition_prefix.merge_with(&other.definition_prefix),
        }
    }
}

/// A context associates config options with a component interface, and provides
/// helper methods that are shared between all templates and filters in this
/// module.
#[derive(Clone, Copy)]
pub struct Context<'config, 'ci> {
    config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> Context<'config, 'ci> {
    /// Creates a new context with options for the given component interface.
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Context { config, ci }
    }

    /// Returns the `RustBuffer` type name.
    ///
    /// A `RustBuffer` is a Plain Old Data struct that holds a pointer to a
    /// Rust byte buffer, along with its length and capacity. Because the
    /// generated binding for each component declares its own FFI symbols in an
    /// `extern "C"` block, the `RustBuffer` type name must be unique for each
    /// component.
    ///
    /// Declaring multiple types with the same name in an `extern "C"` block,
    /// even if they're in different header files, will fail the build because
    /// it violates the One Definition Rule.
    pub fn ffi_rustbuffer_type(&self) -> String {
        format!("{}_RustBuffer", self.ci.ffi_namespace())
    }

    /// Returns the `ForeignBytes` type name.
    ///
    /// `ForeignBytes` is a Plain Old Data struct that holds a pointer to some
    /// memory allocated by C++, along with its length. See the docs for
    /// `ffi_rustbuffer_type` about why this type name must be unique for each
    /// component.
    pub fn ffi_foreignbytes_type(&self) -> String {
        format!("{}_ForeignBytes", self.ci.ffi_namespace())
    }

    /// Returns the `RustError` type name.
    ///
    /// A `RustError` is a Plain Old Data struct that holds an error code and
    /// a message string. See the docs for `ffi_rustbuffer_type` about why this
    /// type name must be unique for each component.
    pub fn ffi_rusterror_type(&self) -> String {
        format!("{}_RustError", self.ci.ffi_namespace())
    }

    /// Returns the name to use for the `detail` C++ namespace, which contains
    /// the serialization helpers and other internal types. This name must be
    /// unique for each component.
    pub fn detail_name(&self) -> String {
        format!("{}_detail", self.ci.namespace())
    }

    /// Returns the unprefixed, unmodified component namespace name. This is
    /// exposed for convenience, where a template has access to the context but
    /// not the component interface.
    pub fn namespace(&self) -> &'ci str {
        self.ci.namespace()
    }

    /// Returns the type name to use for an interface, dictionary, enum, or
    /// namespace with the given `ident` in the generated WebIDL and C++ code.
    pub fn type_name<'a>(&self, ident: &'a str) -> Cow<'a, str> {
        // Prepend the definition prefix if there is one; otherwise, just pass
        // the name back as-is.
        match self.config.definition_prefix.as_ref() {
            Some(prefix) => Cow::Owned(format!("{}{}", prefix, ident)),
            None => Cow::Borrowed(ident),
        }
    }

    /// Returns the C++ header or source file name to use for the given
    /// WebIDL interface or namespace name.
    pub fn header_name(&self, ident: &str) -> String {
        self.type_name(ident).to_camel_case()
    }
}

/// A template for a Firefox WebIDL file. We only generate one of these per
/// component.
#[derive(Template)]
#[template(syntax = "webidl", escape = "none", path = "WebIDLTemplate.webidl")]
pub struct WebIdl<'config, 'ci> {
    context: Context<'config, 'ci>,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> WebIdl<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        let context = Context::new(config, ci);
        Self { context, ci }
    }
}

/// A shared header file that's included by all our bindings. This defines
/// common serialization logic and `extern` declarations for the FFI. These
/// namespace and interface source files `#include` this file.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "SharedHeaderTemplate.h")]
pub struct SharedHeader<'config, 'ci> {
    context: Context<'config, 'ci>,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> SharedHeader<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        let context = Context::new(config, ci);
        Self { context, ci }
    }
}

/// A header file generated for a namespace containing top-level functions. If
/// the namespace in the UniFFI IDL file is empty, this file isn't generated.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "NamespaceHeaderTemplate.h")]
pub struct NamespaceHeader<'config, 'ci> {
    context: Context<'config, 'ci>,
    functions: &'ci [Function],
}

impl<'config, 'ci> NamespaceHeader<'config, 'ci> {
    pub fn new(context: Context<'config, 'ci>, functions: &'ci [Function]) -> Self {
        Self { context, functions }
    }
}

/// An implementation file for a namespace with top-level functions. If the
/// namespace in the UniFFI IDL is empty, this isn't generated.
#[derive(Template)]
#[template(syntax = "cpp", escape = "none", path = "NamespaceTemplate.cpp")]
pub struct Namespace<'config, 'ci> {
    context: Context<'config, 'ci>,
    functions: &'ci [Function],
}

impl<'config, 'ci> Namespace<'config, 'ci> {
    pub fn new(context: Context<'config, 'ci>, functions: &'ci [Function]) -> Self {
        Self { context, functions }
    }
}

/// A header file generated for each interface in the UniFFI IDL.
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "InterfaceHeaderTemplate.h")]
pub struct InterfaceHeader<'config, 'ci> {
    context: Context<'config, 'ci>,
    obj: &'ci Object,
}

impl<'config, 'ci> InterfaceHeader<'config, 'ci> {
    pub fn new(context: Context<'config, 'ci>, obj: &'ci Object) -> Self {
        Self { context, obj }
    }
}

/// An implementation file generated for each interface in the UniFFI IDL.
#[derive(Template)]
#[template(syntax = "cpp", escape = "none", path = "InterfaceTemplate.cpp")]
pub struct Interface<'config, 'ci> {
    context: Context<'config, 'ci>,
    obj: &'ci Object,
}

impl<'config, 'ci> Interface<'config, 'ci> {
    pub fn new(context: Context<'config, 'ci>, obj: &'ci Object) -> Self {
        Self { context, obj }
    }
}

/// Filters for our Askama templates above. These output C++ and WebIDL.
mod filters {
    use super::*;

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
    pub fn type_webidl(type_: &Type, context: &Context<'_, '_>) -> Result<String, askama::Error> {
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
            Type::Enum(name) | Type::Record(name) | Type::Object(name) => {
                class_name_webidl(name, context)?
            }
            Type::Error(_name) => {
                // TODO: We don't currently throw typed errors; see
                // https://github.com/mozilla/uniffi-rs/issues/295.
                panic!("[TODO: type_webidl({:?})]", type_)
            }
            Type::Optional(inner) => format!("{}?", type_webidl(inner, context)?),
            Type::Sequence(inner) => format!("sequence<{}>", type_webidl(inner, context)?),
            Type::Map(inner) => format!("record<DOMString, {}>", type_webidl(inner, context)?),
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
    pub fn type_ffi(type_: &FFIType, context: &Context<'_, '_>) -> Result<String, askama::Error> {
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
            FFIType::RustBuffer => context.ffi_rustbuffer_type(),
            FFIType::RustError => context.ffi_rusterror_type(),
            FFIType::ForeignBytes => context.ffi_foreignbytes_type(),
        })
    }

    /// Declares a C++ type.
    pub fn type_cpp(type_: &Type, context: &Context<'_, '_>) -> Result<String, askama::Error> {
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
            Type::Enum(name) | Type::Record(name) => class_name_cpp(name, context)?,
            Type::Object(name) => format!("OwningNonNull<{}>", class_name_cpp(name, context)?),
            Type::Optional(inner) => {
                // Nullable objects become `RefPtr<T>` (instead of
                // `OwningNonNull<T>`); all others become `Nullable<T>`.
                match inner.as_ref() {
                    Type::Object(name) => format!("RefPtr<{}>", class_name_cpp(name, context)?),
                    Type::String => "nsString".into(),
                    _ => format!("Nullable<{}>", type_cpp(inner, context)?),
                }
            }
            Type::Sequence(inner) => format!("nsTArray<{}>", type_cpp(inner, context)?),
            Type::Map(inner) => format!("Record<nsString, {}>", type_cpp(inner, context)?),
            Type::Error(_name) => {
                // TODO: We don't currently throw typed errors; see
                // https://github.com/mozilla/uniffi-rs/issues/295.
                panic!("[TODO: type_cpp({:?})]", type_)
            }
        })
    }

    fn in_arg_type_cpp(type_: &Type, context: &Context<'_, '_>) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Optional(inner) => match inner.as_ref() {
                Type::Object(_) | Type::String => type_cpp(type_, context)?,
                _ => format!("Nullable<{}>", in_arg_type_cpp(inner, context)?),
            },
            Type::Sequence(inner) => format!("Sequence<{}>", in_arg_type_cpp(&inner, context)?),
            _ => type_cpp(type_, context)?,
        })
    }

    /// Declares a C++ in or out argument type.
    pub fn arg_type_cpp(
        arg: &BindingArgument<'_>,
        context: &Context<'_, '_>,
    ) -> Result<String, askama::Error> {
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
                    Type::Object(name) => format!("{}&", class_name_cpp(&name, context)?),
                    Type::Optional(inner) => match inner.as_ref() {
                        Type::String => "const nsAString&".into(),
                        Type::Object(name) => format!("{}*", class_name_cpp(&name, context)?),
                        _ => format!("const {}&", in_arg_type_cpp(&arg.type_(), context)?),
                    },
                    Type::Record(_) | Type::Map(_) | Type::Sequence(_) => {
                        format!("const {}&", in_arg_type_cpp(&arg.type_(), context)?)
                    }
                    _ => in_arg_type_cpp(&arg.type_(), context)?,
                }
            }
            BindingArgument::Out(type_) => {
                // Out arguments are usually passed by reference. `nsString`
                // becomes `nsAString`.
                match type_ {
                    Type::String => "nsAString&".into(),
                    Type::Optional(inner) => match inner.as_ref() {
                        Type::String => "nsAString&".into(),
                        _ => format!("{}&", type_cpp(type_, context)?),
                    },
                    _ => format!("{}&", type_cpp(type_, context)?),
                }
            }
        })
    }

    /// Declares a C++ return type.
    pub fn ret_type_cpp(type_: &Type, context: &Context<'_, '_>) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Object(name) => format!("already_AddRefed<{}>", class_name_cpp(name, context)?),
            Type::Optional(inner) => match inner.as_ref() {
                Type::Object(name) => {
                    format!("already_AddRefed<{}>", class_name_cpp(name, context)?)
                }
                _ => type_cpp(type_, context)?,
            },
            _ => type_cpp(type_, context)?,
        })
    }

    /// Generates a dummy value for a given return type. A C++ function that
    /// declares a return type must return some value of that type, even if it
    /// throws a DOM exception via the `ErrorResult`.
    pub fn dummy_ret_value_cpp(
        return_type: &Type,
        context: &Context<'_, '_>,
    ) -> Result<String, askama::Error> {
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
            Type::Enum(name) => format!("{}::EndGuard_", class_name_cpp(name, context)?),
            Type::Object(_) => "nullptr".into(),
            Type::String => "EmptyString()".into(),
            Type::Optional(_) | Type::Record(_) | Type::Map(_) | Type::Sequence(_) => {
                format!("{}()", type_cpp(return_type, context)?)
            }
            Type::Error(_) => {
                // TODO: We don't currently throw typed errors; see
                // https://github.com/mozilla/uniffi-rs/issues/295.
                panic!("[TODO: dummy_ret_value_cpp({:?})]", return_type)
            }
        })
    }

    /// Generates an expression for lowering a C++ type into a C type when
    /// calling an FFI function.
    pub fn lower_cpp(
        type_: &Type,
        from: &str,
        context: &Context<'_, '_>,
    ) -> Result<String, askama::Error> {
        let (lifted, nullable) = match type_ {
            // Since our in argument type is `nsAString`, we need to use that
            // to instantiate `ViaFfi`, not `nsString`.
            Type::String => ("nsAString".into(), false),
            Type::Optional(inner) => match inner.as_ref() {
                Type::String => ("nsAString".into(), true),
                _ => (in_arg_type_cpp(type_, context)?, false),
            },
            _ => (in_arg_type_cpp(type_, context)?, false),
        };
        Ok(format!(
            "{}::ViaFfi<{}, {}, {}>::Lower({})",
            context.detail_name(),
            lifted,
            type_ffi(&FFIType::from(type_), context)?,
            nullable,
            from
        ))
    }

    /// Generates an expression for lifting a C return type from the FFI into a
    /// C++ out parameter.
    pub fn lift_cpp(
        type_: &Type,
        from: &str,
        into: &str,
        context: &Context<'_, '_>,
    ) -> Result<String, askama::Error> {
        let (lifted, nullable) = match type_ {
            // Out arguments are also `nsAString`, so we need to use it for the
            // instantiation.
            Type::String => ("nsAString".into(), false),
            Type::Optional(inner) => match inner.as_ref() {
                Type::String => ("nsAString".into(), true),
                _ => (type_cpp(type_, context)?, false),
            },
            _ => (type_cpp(type_, context)?, false),
        };
        Ok(format!(
            "{}::ViaFfi<{}, {}, {}>::Lift({}, {})",
            context.detail_name(),
            lifted,
            type_ffi(&FFIType::from(type_), context)?,
            nullable,
            from,
            into,
        ))
    }

    pub fn var_name_webidl(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_mixed_case())
    }

    pub fn enum_variant_webidl(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_mixed_case())
    }

    pub fn header_name_cpp(nm: &str, context: &Context<'_, '_>) -> Result<String, askama::Error> {
        Ok(context.header_name(nm))
    }

    /// Declares an interface, dictionary, enum, or namespace name in WebIDL.
    pub fn class_name_webidl(nm: &str, context: &Context<'_, '_>) -> Result<String, askama::Error> {
        Ok(context.type_name(nm).to_camel_case())
    }

    /// Declares a class name in C++.
    pub fn class_name_cpp(nm: &str, context: &Context<'_, '_>) -> Result<String, askama::Error> {
        Ok(context.type_name(nm).to_camel_case())
    }

    /// Declares a method name in WebIDL.
    pub fn fn_name_webidl(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Declares a class or instance method name in C++. Function and methods
    /// names are UpperCamelCase in C++, even though they're mixedCamelCase in
    /// WebIDL.
    pub fn fn_name_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// `Codegen.py` emits field names as `mFieldName`. The `m` prefix is Gecko
    /// style for struct members.
    pub fn field_name_cpp(nm: &str) -> Result<String, askama::Error> {
        Ok(format!("m{}", nm.to_camel_case()))
    }

    pub fn enum_variant_cpp(nm: &str) -> Result<String, askama::Error> {
        // TODO: Make sure this does the right thing for hyphenated variants
        // (https://github.com/mozilla/uniffi-rs/issues/294), or the generated
        // code won't compile.
        //
        // Example: "bookmark-added" should become `Bookmark_added`, because
        // that's what Firefox's `Codegen.py` spits out.
        Ok(nm.to_camel_case())
    }
}
