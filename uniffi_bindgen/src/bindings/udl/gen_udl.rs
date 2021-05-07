/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;

use crate::interface::*;

#[derive(Template)]
#[template(syntax = "udl", escape = "none", path = "component.udl")]
pub struct UDLFile<'a> {
    ci: &'a ComponentInterface,
}
impl<'a> UDLFile<'a> {
    pub fn new(ci: &'a ComponentInterface) -> Self {
        Self { ci }
    }
}

mod filters {
    use super::*;

    /// Get the UDL syntax for representing a given api-level `Type`.
    pub fn type_udl(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            Type::UInt8 => "u8".to_string(),
            Type::UInt16 => "u16".to_string(),
            Type::UInt32 => "u32".to_string(),
            Type::UInt64 => "u64".to_string(),
            Type::Int8 => "i8".to_string(),
            Type::Int16 => "i16".to_string(),
            Type::Int32 => "i32".to_string(),
            Type::Int64 => "i64".to_string(),
            Type::Float32 => "f32".to_string(),
            Type::Float64 => "f64".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::String => "string".to_string(),
            Type::Enum(name)
            | Type::Record(name)
            | Type::Object(name)
            | Type::Error(name)
            | Type::CallbackInterface(name) => name.clone(),
            Type::Optional(t) => format!("{}?", type_udl(t)?),
            Type::Sequence(t) => format!("sequence<{}>", type_udl(t)?),
            Type::Map(t) => format!("record<DOMString, {}>", type_udl(t)?),
        })
    }

    pub fn return_type_udl(type_: &Option<&Type>) -> Result<String, askama::Error> {
        Ok(match type_ {
            None => "void".to_string(),
            Some(t) => type_udl(t)?,
        })
    }

    pub fn docstring(docs: &Vec<&str>, indent: &usize) -> Result<String, askama::Error> {
        let mut docstr = String::new();
        let indent = " ".repeat(*indent);
        for ln in docs {
            docstr.push_str("\n");
            docstr.push_str(indent.as_str());
            docstr.push_str("//");
            docstr.push_str(ln);
        }
        Ok(docstr)
    }
}
