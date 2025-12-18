/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use std::borrow::Borrow;

use super::interface::*;
use heck::ToShoutySnakeCase;

#[derive(Template)]
#[template(syntax = "rs", escape = "none", path = "scaffolding_template.rs")]
pub struct RustScaffolding<'a> {
    ci: &'a ComponentInterface,
    udl_base_name: &'a str,
    pointer_ffi: bool,
}
impl<'a> RustScaffolding<'a> {
    pub fn new(ci: &'a ComponentInterface, udl_base_name: &'a str) -> Self {
        Self {
            ci,
            udl_base_name,
            pointer_ffi: cfg!(feature = "pointer-ffi"),
        }
    }
}
mod filters {
    use super::*;

    pub fn pointer_ffi_symbol_name(
        name: &str,
        _values: &dyn askama::Values,
    ) -> Result<String, askama::Error> {
        Ok(uniffi_meta::pointer_ffi_symbol_name(name))
    }

    pub fn type_rs(type_: &Type, _values: &dyn askama::Values) -> Result<String, askama::Error> {
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
            Type::String => "::std::string::String".into(),
            Type::Bytes => "::std::vec::Vec<u8>".into(),
            Type::Timestamp => "::std::time::SystemTime".into(),
            Type::Duration => "::std::time::Duration".into(),
            Type::Enum { name, .. } | Type::Record { name, .. } => format!("r#{name}"),
            Type::Object { name, imp, .. } => {
                format!("::std::sync::Arc<{}>", imp.rust_name_for(name))
            }
            Type::CallbackInterface { name, .. } => format!("Box<dyn r#{name}>"),
            Type::Optional { inner_type } => {
                format!("::std::option::Option<{}>", type_rs(inner_type, _values)?)
            }
            Type::Sequence { inner_type } => {
                format!("std::vec::Vec<{}>", type_rs(inner_type, _values)?)
            }
            Type::Map {
                key_type,
                value_type,
            } => format!(
                "::std::collections::HashMap<{}, {}>",
                type_rs(key_type, _values)?,
                type_rs(value_type, _values)?
            ),
            Type::Custom { name, .. } => format!("r#{name}"),
        })
    }
}
