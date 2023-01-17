/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use std::borrow::Borrow;

use super::interface::*;
use heck::ToSnakeCase;

#[derive(Template)]
#[template(syntax = "rs", escape = "none", path = "scaffolding_template.rs")]
pub struct RustScaffolding<'a> {
    ci: &'a ComponentInterface,
    uniffi_version: &'static str,
}
impl<'a> RustScaffolding<'a> {
    pub fn new(ci: &'a ComponentInterface) -> Self {
        Self {
            ci,
            uniffi_version: crate::BINDGEN_VERSION,
        }
    }
}
mod filters {
    use super::*;

    pub fn type_rs(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8 => "i8".into(),
            Type::UInt8 => "u8".into(),
            Type::Int16 => "i16".into(),
            Type::UInt16 => "u16".into(),
            Type::Int32 => "i32".into(),
            Type::UInt32 => "u32".into(),
            Type::Int64 => "i64".into(),
            Type::UInt64 => "u64".into(),
            Type::Float32 => "f32".into(),
            Type::Float64 => "f64".into(),
            Type::Boolean => "bool".into(),
            Type::String => "String".into(),
            Type::Timestamp => "std::time::SystemTime".into(),
            Type::Duration => "std::time::Duration".into(),
            Type::Enum(name) | Type::Record(name) | Type::Error(name) => format!("r#{name}"),
            Type::Object(name) => format!("std::sync::Arc<r#{name}>"),
            Type::CallbackInterface(name) => format!("Box<dyn r#{name}>"),
            Type::Optional(t) => format!("std::option::Option<{}>", type_rs(t)?),
            Type::Sequence(t) => format!("std::vec::Vec<{}>", type_rs(t)?),
            Type::Map(k, v) => format!(
                "std::collections::HashMap<{}, {}>",
                type_rs(k)?,
                type_rs(v)?
            ),
            Type::Custom { name, .. } => format!("r#{name}"),
            Type::External { .. } => panic!("External types coming to a uniffi near you soon!"),
            Type::Unresolved { .. } => {
                unreachable!("UDL scaffolding code never contains unresolved types")
            }
        })
    }

    pub fn type_ffi(type_: &FfiType) -> Result<String, askama::Error> {
        Ok(match type_ {
            FfiType::Int8 => "i8".into(),
            FfiType::UInt8 => "u8".into(),
            FfiType::Int16 => "i16".into(),
            FfiType::UInt16 => "u16".into(),
            FfiType::Int32 => "i32".into(),
            FfiType::UInt32 => "u32".into(),
            FfiType::Int64 => "i64".into(),
            FfiType::UInt64 => "u64".into(),
            FfiType::Float32 => "f32".into(),
            FfiType::Float64 => "f64".into(),
            FfiType::RustArcPtr(_) => "*const std::os::raw::c_void".into(),
            FfiType::RustBuffer(_) => "uniffi::RustBuffer".into(),
            FfiType::ForeignBytes => "uniffi::ForeignBytes".into(),
            FfiType::ForeignCallback => "uniffi::ForeignCallback".into(),
        })
    }

    /// Get the name of the FfiConverter implementation for this type
    ///
    /// - For primitives / standard types this is the type itself.
    /// - For user-defined types, this is a unique generated name.  We then generate a unit-struct
    ///   in the scaffolding code that implements FfiConverter.
    pub fn ffi_converter_name(type_: &Type) -> askama::Result<String> {
        Ok(match type_ {
            // Timestamp/Duraration are handled by standard types
            Type::Timestamp => "std::time::SystemTime".into(),
            Type::Duration => "std::time::Duration".into(),
            // Object is handled by Arc<T>
            Type::Object(name) => format!("std::sync::Arc<r#{name}>"),
            // Other user-defined types are handled by a unit-struct that we generate.  The
            // FfiConverter implementation for this can be found in one of the scaffolding template code.
            //
            // We generate a unit-struct to sidestep Rust's orphan rules (ADR-0006).
            //
            // CallbackInterface is handled by special case code on both the scaffolding and
            // bindings side.  It's not a unit-struct, but the same name generation code works.
            Type::Enum(_) | Type::Record(_) | Type::Error(_) | Type::CallbackInterface(_) => {
                format!("FfiConverter{}", type_.canonical_name())
            }
            // Wrapper types are implemented by generics that wrap the FfiConverter implementation of the
            // inner type.
            Type::Optional(inner) => {
                format!("std::option::Option<{}>", ffi_converter_name(inner)?)
            }
            Type::Sequence(inner) => format!("std::vec::Vec<{}>", ffi_converter_name(inner)?),
            Type::Map(k, v) => format!(
                "std::collections::HashMap<{}, {}>",
                ffi_converter_name(k)?,
                ffi_converter_name(v)?
            ),
            // External and Wrapped bytes have FfiConverters with a predictable name based on the type name.
            Type::Custom { name, .. } | Type::External { name, .. } => {
                format!("FfiConverterType{name}")
            }
            // Primitive types / strings are implemented by their rust type
            Type::Int8 => "i8".into(),
            Type::UInt8 => "u8".into(),
            Type::Int16 => "i16".into(),
            Type::UInt16 => "u16".into(),
            Type::Int32 => "i32".into(),
            Type::UInt32 => "u32".into(),
            Type::Int64 => "i64".into(),
            Type::UInt64 => "u64".into(),
            Type::Float32 => "f32".into(),
            Type::Float64 => "f64".into(),
            Type::String => "String".into(),
            Type::Boolean => "bool".into(),
            Type::Unresolved { .. } => {
                unreachable!("UDL scaffolding code never contains unresolved types")
            }
        })
    }

    // Map a type to Rust code that specifies the FfiConverter implementation.
    //
    // This outputs something like `<TheFfiConverterStruct as FfiConverter>`
    pub fn ffi_converter(type_: &Type) -> Result<String, askama::Error> {
        Ok(format!(
            "<{} as uniffi::FfiConverter>",
            ffi_converter_name(type_)?
        ))
    }

    // Turns a `crate-name` into the `crate_name` the .rs code needs to specify.
    pub fn crate_name_rs(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }
}
