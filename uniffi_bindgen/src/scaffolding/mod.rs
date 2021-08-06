/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;

mod type_logic;

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
        Ok(type_logic::rust_type(type_))
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

    pub fn ffi_converter_name(type_: &Type) -> askama::Result<String> {
        Ok(type_logic::ffi_converter_name(type_))
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
}
