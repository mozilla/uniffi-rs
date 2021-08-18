/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::interface::*;
use crate::MergeWith;
use crate::UniffiContext;

// Some config options for it the caller wants to customize the generated python.
// Note that this can only be used to control details of the python *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    cdylib_name: Option<String>,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
        }
    }
}

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
pub struct PythonWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    context: &'a UniffiContext,
}
impl<'a> PythonWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface, context: &'a UniffiContext) -> Self {
        Self { config, ci, context}
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
            FFIType::ExternalRustBuffer { crate_name } => {
                format!("{}.RustBuffer", mod_name_py(crate_name)?)
            }
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

    pub fn mod_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn var_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    pub fn enum_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    pub fn ffi_converter_name(type_: &Type) -> Result<String, askama::Error> {
        Ok(format!("FfiConverter{}", type_.canonical_name().to_camel_case()))
    }

    pub fn python_wrapper_name(type_: &Type) -> Result<Option<String>, askama::Error> {
        Ok(match type_ {
            Type::Wrapped { name, languages, .. } => {
                if languages.contains(&Language::Python) {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None
        })
    }

    pub fn python_wrapper(type_: &Type, context: &UniffiContext) -> Result<Option<String>, askama::Error> {
        Ok(match type_ {
            Type::Wrapped { name, languages, .. } => {
                if languages.contains(&Language::Python) {
                    let path = context.get_bindings_path("python", format!("{}.py", name));
                    Some(fs::read_to_string(&path).expect(
                            &format!("Error reading wrapper file: {:?}", &path)
                    ))
                } else {
                    None
                }
            }
            _ => None
        })
    }
}
