/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;

use super::interface::*;

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
            Type::Enum(name) | Type::Record(name) | Type::Error(name) => name.clone(),
            Type::Object(name) => format!("std::sync::Arc<{}>", name),
            Type::CallbackInterface(name) => format!("Box<dyn {}>", name),
            Type::Optional(t) => format!("Option<{}>", type_rs(t)?),
            Type::Sequence(t) => format!("Vec<{}>", type_rs(t)?),
            Type::Map(t) => format!("std::collections::HashMap<String, {}>", type_rs(t)?),
        })
    }

    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            FFIType::Int8 => "i8".into(),
            FFIType::UInt8 => "u8".into(),
            FFIType::Int16 => "i16".into(),
            FFIType::UInt16 => "u16".into(),
            FFIType::Int32 => "i32".into(),
            FFIType::UInt32 => "u32".into(),
            FFIType::Int64 => "i64".into(),
            FFIType::UInt64 => "u64".into(),
            FFIType::Float32 => "f32".into(),
            FFIType::Float64 => "f64".into(),
            FFIType::RustArcPtr => "*const std::os::raw::c_void".into(),
            FFIType::RustBuffer => "uniffi::RustBuffer".into(),
            FFIType::ForeignBytes => "uniffi::ForeignBytes".into(),
            FFIType::ForeignCallback => "uniffi::ForeignCallback".into(),
        })
    }

    pub fn as_viaffi(type_: &Type) -> Result<String, askama::Error> {
        Ok(format!("<{} as uniffi::ViaFfi>", type_.viaffi_impl_name()))
    }
}
