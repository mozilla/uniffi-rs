/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::fmt;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use serde::{Deserialize, Serialize};

use crate::backend::{CodeDeclaration, CodeOracle, CodeType, TypeIdentifier};
use crate::interface::*;
use crate::MergeWith;

mod compounds;
mod enum_;
mod error;
mod external;
mod function;
mod miscellany;
mod object;
mod primitives;
mod record;
mod wrapped;

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
    oracle: PythonCodeOracle,
}
impl<'a> PythonWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self {
            config,
            ci,
            oracle: Default::default(),
        }
    }

    pub fn members(&self) -> Vec<Box<dyn CodeDeclaration + 'a>> {
        let ci = self.ci;
        vec![]
            .into_iter()
            .chain(ci.iter_enum_definitions().into_iter().map(|inner| {
                Box::new(enum_::PythonEnum::new(inner, ci)) as Box<dyn CodeDeclaration>
            }))
            .chain(ci.iter_function_definitions().into_iter().map(|inner| {
                Box::new(function::PythonFunction::new(inner, ci)) as Box<dyn CodeDeclaration>
            }))
            .chain(ci.iter_object_definitions().into_iter().map(|inner| {
                Box::new(object::PythonObject::new(inner, ci)) as Box<dyn CodeDeclaration>
            }))
            .chain(ci.iter_record_definitions().into_iter().map(|inner| {
                Box::new(record::PythonRecord::new(inner, ci)) as Box<dyn CodeDeclaration>
            }))
            .chain(ci.iter_error_definitions().into_iter().map(|inner| {
                Box::new(error::PythonError::new(inner, ci)) as Box<dyn CodeDeclaration>
            }))
            .collect()
    }

    pub fn initialization_code(&self) -> Vec<String> {
        let oracle = &self.oracle;
        self.members()
            .into_iter()
            .filter_map(|member| member.initialization_code(oracle))
            .collect()
    }

    pub fn declaration_code(&self) -> Vec<String> {
        let oracle = &self.oracle;
        self.members()
            .into_iter()
            .filter_map(|member| member.definition_code(oracle))
            .chain(
                self.ci
                    .iter_types()
                    .into_iter()
                    .filter_map(|type_| oracle.find(&type_).helper_code(oracle)),
            )
            .collect()
    }

    pub fn imports(&self) -> Vec<String> {
        let oracle = &self.oracle;
        let mut imports: Vec<String> = self
            .members()
            .into_iter()
            .filter_map(|member| member.imports(oracle))
            .flatten()
            .chain(
                self.ci
                    .iter_types()
                    .into_iter()
                    .filter_map(|type_| oracle.find(&type_).imports(oracle))
                    .flatten(),
            )
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();

        imports.sort();
        imports
    }
}

#[derive(Default)]
pub struct PythonCodeOracle;

impl PythonCodeOracle {
    fn create_code_type(&self, type_: TypeIdentifier) -> Box<dyn CodeType> {
        // I really want access to the ComponentInterface here so I can look up the interface::{Enum, Record, Error, Object, etc}
        // However, there's some violence and gore I need to do to (temporarily) make the oracle usable from filters.

        // Some refactor of the templates is needed to make progress here: I think most of the filter functions need to take an &dyn CodeOracle
        match type_ {
            Type::UInt8 => Box::new(primitives::UInt8CodeType),
            Type::Int8 => Box::new(primitives::Int8CodeType),
            Type::UInt16 => Box::new(primitives::UInt16CodeType),
            Type::Int16 => Box::new(primitives::Int16CodeType),
            Type::UInt32 => Box::new(primitives::UInt32CodeType),
            Type::Int32 => Box::new(primitives::Int32CodeType),
            Type::UInt64 => Box::new(primitives::UInt64CodeType),
            Type::Int64 => Box::new(primitives::Int64CodeType),
            Type::Float32 => Box::new(primitives::Float32CodeType),
            Type::Float64 => Box::new(primitives::Float64CodeType),
            Type::Boolean => Box::new(primitives::BooleanCodeType),
            Type::String => Box::new(primitives::StringCodeType),

            Type::Timestamp => Box::new(miscellany::TimestampCodeType),
            Type::Duration => Box::new(miscellany::DurationCodeType),

            Type::Enum(id) => Box::new(enum_::EnumCodeType::new(id)),
            Type::Object(id) => Box::new(object::ObjectCodeType::new(id)),
            Type::Record(id) => Box::new(record::RecordCodeType::new(id)),
            Type::Error(id) => Box::new(error::ErrorCodeType::new(id)),
            Type::CallbackInterface(_id) => {
                unimplemented!("Callback interfaces are not implemented")
            }

            Type::Optional(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::OptionalCodeType::new(inner, outer))
            }
            Type::Sequence(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::SequenceCodeType::new(inner, outer))
            }
            Type::Map(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::MapCodeType::new(inner, outer))
            }
            Type::External { name, crate_name } => {
                Box::new(external::ExternalCodeType::new(name, crate_name))
            }
            Type::Wrapped { ref prim, .. } => {
                let outer = type_.clone();
                let inner = *prim.to_owned();
                Box::new(wrapped::WrappedCodeType::new(inner, outer))
            }
        }
    }
}

impl CodeOracle for PythonCodeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Box<dyn CodeType> {
        self.create_code_type(type_.clone())
    }

    /// Get the idiomatic Python rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_camel_case()
    }

    /// Get the idiomatic Python rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_snake_case()
    }

    /// Get the idiomatic Python rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_snake_case()
    }

    /// Get the idiomatic Python rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_shouty_snake_case()
    }

    /// Get the idiomatic Python rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn error_name(&self, nm: &dyn fmt::Display) -> String {
        let name = nm.to_string();
        match name.strip_suffix("Error") {
            None => name,
            Some(stripped) => {
                let mut py_exc_name = stripped.to_owned();
                py_exc_name.push_str("Exception");
                py_exc_name
            }
        }
    }

    fn ffi_type_label(&self, ffi_type: &FFIType) -> String {
        match ffi_type {
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
        }
    }
}

pub mod filters {
    use super::*;
    use std::fmt;

    fn oracle() -> impl CodeOracle {
        PythonCodeOracle
    }

    pub fn type_py(type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).type_label(&oracle))
    }

    pub fn canonical_name(type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).canonical_name(&oracle))
    }

    pub fn lower_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).lower(&oracle, nm))
    }

    pub fn write_py(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).write(&oracle, nm, target))
    }

    pub fn lift_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).lift(&oracle, nm))
    }

    pub fn literal_py(literal: &Literal, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).literal(&oracle, literal))
    }

    pub fn read_py(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_).read(&oracle, nm))
    }

    /// Get the Python syntax for representing a given low-level `FFIType`.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(oracle().ffi_type_label(type_))
    }

    /// Get the idiomatic Python rendering of a class name (for enums, records, errors, etc).
    pub fn class_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Python rendering of a function name.
    pub fn fn_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().fn_name(nm))
    }

    /// Get the idiomatic Python rendering of a variable name.
    pub fn var_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().var_name(nm))
    }

    /// Get the idiomatic Python rendering of an individual enum variant.
    pub fn enum_variant_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().enum_variant_name(nm))
    }

    /// Get the idiomatic Python rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name_py(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().error_name(nm))
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
            Type::Wrapped { prim, .. } => coerce_py(nm, prim.as_ref())?,
            Type::External { .. } => panic!("should not be necessary to coerce External types"),
        })
    }
}
