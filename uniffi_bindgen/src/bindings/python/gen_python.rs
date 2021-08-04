/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

// Some config options for it the caller wants to customize the generated python.
// Note that this can only be used to control details of the python *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    cdylib_name: Option<String>,
    module_name: Option<String>,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn module_name(&self, default: &str) -> String {
        match &self.module_name {
            Some(module_name) => module_name.clone(),
            None => default.to_string(),
        }
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
            module_name: Some(ci.namespace().to_string()),
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
            module_name: self.module_name.merge_with(&other.module_name),
        }
    }
}

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
pub struct PythonWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> PythonWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            FFIType::Int8 => "ctypes.c_int8".to_string(),
            FFIType::UInt8 => "ctypes.c_uint8".to_string(),
            FFIType::Int16 => "ctypes.c_int16".to_string(),
            FFIType::UInt16 => "ctypes.c_uint16".to_string(),
            FFIType::Int32 => "ctypes.c_int32".to_string(),
            FFIType::UInt32 => "ctypes.c_uint32".to_string(),
            FFIType::Int64 => "ctypes.c_int64".to_string(),
            FFIType::UInt64 => "ctypes.c_uint64".to_string(),
            FFIType::Float32 => "ctypes.c_float".to_string(),
            FFIType::Float64 => "ctypes.c_double".to_string(),
            FFIType::RustArcPtr => "ctypes.c_void_p".to_string(),
            FFIType::RustBuffer => "RustBuffer".to_string(),
            FFIType::ForeignBytes => "ForeignBytes".to_string(),
            FFIType::ForeignCallback => unimplemented!("Callback interfaces are not implemented"),
        })
    }

    pub fn literal_py(literal: &Literal) -> Result<String, askama::Error> {
        Ok(match literal {
            Literal::Boolean(v) => {
                if *v {
                    "True".into()
                } else {
                    "False".into()
                }
            }
            // use the double-quote form to match with the other languages, and quote escapes.
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "None".into(),
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "{}".into(),
            Literal::Enum(v, type_) => match type_ {
                Type::Enum(name) => format!("{}.{}", class_name_py(name)?, enum_name_py(v)?),
                _ => panic!("Unexpected type in enum literal: {:?}", type_),
            },
            // https://docs.python.org/3/reference/lexical_analysis.html#integer-literals
            Literal::Int(i, radix, _) => match radix {
                Radix::Octal => format!("0o{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
            Literal::UInt(i, radix, _) => match radix {
                Radix::Octal => format!("0o{:o}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
            Literal::Float(string, _type_) => string.clone(),
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

    pub fn coerce_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => format!("int({})", nm), // TODO: check max/min value
            Type::Float32 | Type::Float64 => format!("float({})", nm),
            Type::Boolean => format!("bool({})", nm),
            Type::String
            | Type::Object(_)
            | Type::Imported(..)
            | Type::Enum(_)
            | Type::Error(_)
            | Type::Record(_)
            | Type::Timestamp
            | Type::Duration => nm.to_string(),
            Type::CallbackInterface(_) => panic!("No support for coercing callback interfaces yet"),
            Type::Optional(t) => format!("(None if {} is None else {})", nm, coerce_py(nm, t)?),
            Type::Sequence(t) => format!("list({} for x in {})", coerce_py(&"x", t)?, nm),
            Type::Map(t) => format!(
                "dict(({},{}) for (k, v) in {}.items())",
                coerce_py(&"k", &Type::String)?,
                coerce_py(&"v", t)?,
                nm
            ),
        })
    }

    pub fn lower_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
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
            | Type::Float64 => nm.to_string(),
            Type::Boolean => format!("(1 if {} else 0)", nm),
            Type::String => format!("RustBuffer.allocFromString({})", nm),
            Type::Object(_) => format!("({}._pointer)", nm),
            Type::Imported(..) => panic!("No support for lowering imported types yet"),
            Type::CallbackInterface(_) => panic!("No support for lowering callback interfaces yet"),
            Type::Error(_) => panic!("No support for lowering errors, yet"),
            Type::Enum(_)
            | Type::Record(_)
            | Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => format!(
                "RustBuffer.allocFrom{}({})",
                class_name_py(&type_.canonical_name())?,
                nm
            ),
        })
    }

    pub fn lift_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => format!("int({})", nm),
            Type::Float32 | Type::Float64 => format!("float({})", nm),
            Type::Boolean => format!("(True if {} else False)", nm),
            Type::String => format!("{}.consumeIntoString()", nm),
            Type::Object(name) => format!("{}._make_instance_({})", class_name_py(name)?, nm),
            Type::CallbackInterface(_) => panic!("No support for lifting callback interfaces, yet"),
            Type::Error(_) => panic!("No support for lowering errors, yet"),
            Type::Imported(..) => panic!("No support for lowering errors, yet"),
            Type::Enum(_)
            | Type::Record(_)
            | Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => format!(
                "{}.consumeInto{}()",
                nm,
                class_name_py(&type_.canonical_name())?
            ),
        })
    }
}
