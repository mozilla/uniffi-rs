/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};

use crate::interface::*;

// Some config options for it the caller wants to customize the generated python.
// Note that this can only be used to control details of the python *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
pub struct Config {
    // No config options yet.
}

impl Config {
    pub fn from(_ci: &ComponentInterface) -> Self {
        Config {
            // No config options yet
        }
    }
}

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
pub struct PythonWrapper<'a> {
    _config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> PythonWrapper<'a> {
    pub fn new(_config: Config, ci: &'a ComponentInterface) -> Self {
        Self { _config, ci }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    pub fn type_c(type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => "ctypes.c_uint32".to_string(),
            TypeReference::U64 => "ctypes.c_uint64".to_string(),
            TypeReference::Float => "ctypes.c_float".to_string(),
            TypeReference::Double => "ctypes.c_double".to_string(),
            TypeReference::Boolean => "ctypes.c_byte".to_string(),
            // We use a c_void_p instead of a c_char_p since python seems to
            // create it's own string if we use c_char_p, and that prevents us
            // from freeing. I could be wrong, but that's what I got from this:
            // https://stackoverflow.com/questions/13445568/python-ctypes-how-to-free-memory-getting-invalid-pointer-error
            TypeReference::String | TypeReference::RawStringPointer => {
                "ctypes.c_void_p".to_string()
            }
            TypeReference::Bytes => "RustBuffer".to_string(),
            TypeReference::Enum(_) => "ctypes.c_uint32".to_string(),
            TypeReference::Record(_) => "RustBuffer".to_string(),
            TypeReference::Optional(_) => "RustBuffer".to_string(),
            TypeReference::Object(_) => "ctypes.c_uint64".to_string(),
            _ => panic!("[TODO: type_c({:?})", type_),
        })
    }

    pub fn class_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    pub fn fn_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn var_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn enum_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    pub fn coerce_py(
        nm: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32
            | TypeReference::U64
            | TypeReference::Float
            | TypeReference::Double
            | TypeReference::String
            | TypeReference::RawStringPointer
            | TypeReference::Boolean => format!("{} = {}", nm, nm),
            TypeReference::Enum(type_name) => format!("{} = {}({})", nm, type_name, nm),
            TypeReference::Record(type_name) => format!("{} = {}._coerce({})", nm, type_name, nm),
            //TypeReference::Optional(_) => "RustBuffer".to_string(),
            _ => panic!("[TODO: coerce_py({:?})]", type_),
        })
    }

    pub fn lower_py(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32
            | TypeReference::U64
            | TypeReference::Float
            | TypeReference::Double
            | TypeReference::Boolean => nm.to_string(),
            TypeReference::Enum(_) => format!("{}.value", nm),
            TypeReference::String => format!("{}.encode('utf-8')", nm),
            TypeReference::Record(type_name) => format!("{}._lower({})", type_name, nm),
            TypeReference::Optional(_type) => format!(
                "lowerOptional({}, lambda buf, v: {})",
                nm,
                lower_into_py(&"buf", &"v", type_)?
            ),
            _ => panic!("[TODO: lower_py({:?})]", type_),
        })
    }

    pub fn lowers_into_size_py(
        nm: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => "4".to_string(),
            TypeReference::Double => "8".to_string(),
            TypeReference::String => format!("4 + len({}.encode('utf-8'))", nm),
            TypeReference::Record(type_name) => format!("{}._lowersIntoSize({})", type_name, nm),
            _ => panic!("[TODO: lowers_into_size_py({:?})]", type_),
        })
    }
    pub fn lower_into_py(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::Double => format!("{}.putDouble({})", target, nm),
            TypeReference::U32 => format!("{}.putInt({})", target, nm),
            TypeReference::String => format!("{}.putString({})", target, nm),
            TypeReference::Record(type_name) => {
                format!("{}._lowerInto({}, {})", type_name, nm, target)
            }
            _ => panic!("[TODO: lower_into_py({:?})]", type_),
        })
    }

    pub fn lift_py(nm: &dyn fmt::Display, type_: &TypeReference) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32
            | TypeReference::U64
            | TypeReference::Float
            | TypeReference::Double
            | TypeReference::Boolean => format!("{}", nm),
            TypeReference::Enum(type_name) => format!("{}({})", type_name, nm),
            TypeReference::String => format!("liftString({})", nm),
            TypeReference::Record(type_name) => format!("{}._lift({})", type_name, nm),
            TypeReference::Optional(type_) => format!(
                "liftOptional({}, lambda buf: {})",
                nm,
                lift_from_py(&"buf", type_)?
            ),
            _ => panic!("[TODO: lift_py({:?})]", type_),
        })
    }

    pub fn lift_from_py(
        nm: &dyn fmt::Display,
        type_: &TypeReference,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            TypeReference::U32 => format!("{}.getInt()", nm),
            TypeReference::Double => format!("{}.getDouble()", nm),
            TypeReference::Record(type_name) => format!("{}._liftFrom({})", type_name, nm),
            TypeReference::String => format!("{}.getString()", nm),
            _ => panic!("[TODO: lift_from_py({:?})]", type_),
        })
    }
}
