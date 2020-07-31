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
}
impl<'a> RustScaffolding<'a> {
    pub fn new(ci: &'a ComponentInterface) -> Self {
        Self { ci }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    pub fn type_rs(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => "u32".to_string(),
            TypeReference::U64 => "u64".to_string(),
            TypeReference::Float => "f32".to_string(),
            TypeReference::Double => "f64".to_string(),
            TypeReference::Boolean => "bool".to_string(),
            TypeReference::String => "&str".to_string(),
            TypeReference::Enum(name) => name.clone(),
            TypeReference::Record(name) => name.clone(),

            // TODO: Clean this up, the `ret_type_rs` is supposed to only be used on return types
            // and solve the issue of asymmetry with strings.
            // When a string is a record member or a part of a sequence, we use the `String` type even if the string is an argument
            // However, when we use strings directly, we use an &str as an argument.
            // This reeks of future issues, a possible solution is to abandon the asymmetry (and FfiStr) and just use
            // `String` everywhere as a ByteBuffer and give up an extra memory allocation
            TypeReference::Optional(t) => format!("Option<{}>", ret_type_rs(t)?),
            TypeReference::Sequence(t) => format!("Vec<{}>", ret_type_rs(t)?),
            _ => panic!("[TODO: type_rs({:?})]", type_),
        })
    }

    pub fn ret_type_rs(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => "u32".to_string(),
            TypeReference::U64 => "u64".to_string(),
            TypeReference::Float => "f32".to_string(),
            TypeReference::Double => "f64".to_string(),
            TypeReference::Boolean => "bool".to_string(),
            TypeReference::String => "String".to_string(),
            TypeReference::Enum(name) => name.clone(),
            TypeReference::Record(name) => name.clone(),
            TypeReference::Optional(t) => format!("Option<{}>", ret_type_rs(t)?),
            TypeReference::Sequence(t) => format!("Vec<{}>", ret_type_rs(t)?),
            _ => panic!("[TODO: ret_type_rs({:?})]", type_),
        })
    }

    pub fn type_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            // Objects don't currently impl `ViaFfi`.
            TypeReference::Object(_) => "u64".to_string(),
            _ => format!("<{} as uniffi::support::ViaFfi>::Value", type_rs(type_)?,),
        })
    }

    pub fn ret_type_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            // Objects don't currently impl `ViaFfi`.
            TypeReference::Object(_) => "u64".to_string(),
            _ => format!(
                "<{} as uniffi::support::ViaFfi>::Value",
                ret_type_rs(type_)?,
            ),
        })
    }

    pub fn lower_rs(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        // By explicitly naming the type here, we help the rust compiler to type-check the user-provided
        // implementations of the functions that we're wrapping (and also to type-check our generated code).
        Ok(format!(
            "<{} as uniffi::support::ViaFfi>::into_ffi_value(&{})",
            ret_type_rs(type_)?,
            nm
        ))
    }

    pub fn lift_rs(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        // By explicitly naming the type here, we help the rust compiler to type-check the user-provided
        // implementations of the functions that we're wrapping (and also to type-check our generated code).
        Ok(format!(
            "<{} as uniffi::support::ViaFfi>::try_from_ffi_value({}).unwrap()",
            type_rs(type_)?,
            nm
        )) // Error handling later...
    }
}
