/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use serde::{Deserialize, Serialize};

use crate::backend::{CodeDeclaration, CodeOracle, CodeType, TemplateExpression, TypeIdentifier};
use crate::interface::*;
use crate::MergeWith;

mod callback_interface;
mod compounds;
mod custom;
mod enum_;
mod error;
mod external;
mod function;
mod miscellany;
mod object;
mod primitives;
mod record;

// config options to customize the generated Kotlin.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    package_name: Option<String>,
    cdylib_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
    internalize: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CustomTypeConfig {
    imports: Option<Vec<String>>,
    type_name: Option<String>,
    into_custom: TemplateExpression,
    from_custom: TemplateExpression,
}

impl Config {
    pub fn package_name(&self) -> String {
        if let Some(package_name) = &self.package_name {
            package_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn internalize(&self) -> bool {
        if let Some(internalize) = &self.internalize {
            internalize.clone()
        } else {
            false
        }
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(format!("uniffi.{}", ci.namespace())),
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
            custom_types: HashMap::new(),
            external_packages: HashMap::new(),
            internalize: Some(false)
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            package_name: self.package_name.merge_with(&other.package_name),
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
            custom_types: self.custom_types.merge_with(&other.custom_types),
            external_packages: self.external_packages.merge_with(&other.external_packages),
            internalize: self.internalize.merge_with(&other.internalize)
        }
    }
}

// Generate kotlin bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    let oracle = KotlinCodeOracle::new(config.clone());
    filters::set_oracle(oracle.clone());

    KotlinWrapper::new(oracle, config.clone(), ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "wrapper.kt")]
pub struct KotlinWrapper<'a> {
    oracle: KotlinCodeOracle,
    config: Config,
    ci: &'a ComponentInterface,
    internalize: bool
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(oracle: KotlinCodeOracle, config: Config, ci: &'a ComponentInterface) -> Self {
        let internalize = config.internalize();
        Self { oracle, config, ci, internalize }
    }

    pub fn members(&self) -> Vec<Box<dyn CodeDeclaration + 'a>> {
        let ci = self.ci;
        vec![
            Box::new(object::KotlinObjectRuntime::new(ci, self.internalize)) as Box<dyn CodeDeclaration>,
            Box::new(callback_interface::KotlinCallbackInterfaceRuntime::new(ci))
                as Box<dyn CodeDeclaration>,
        ]
        .into_iter()
        .chain(
            ci.iter_enum_definitions().into_iter().map(|inner| {
                Box::new(enum_::KotlinEnum::new(inner, ci, self.internalize)) as Box<dyn CodeDeclaration>
            }),
        )
        .chain(ci.iter_function_definitions().into_iter().map(|inner| {
            Box::new(function::KotlinFunction::new(inner, ci, self.internalize)) as Box<dyn CodeDeclaration>
        }))
        .chain(ci.iter_object_definitions().into_iter().map(|inner| {
            Box::new(object::KotlinObject::new(inner, ci, self.internalize)) as Box<dyn CodeDeclaration>
        }))
        .chain(ci.iter_record_definitions().into_iter().map(|inner| {
            Box::new(record::KotlinRecord::new(inner, ci, self.internalize)) as Box<dyn CodeDeclaration>
        }))
        .chain(
            ci.iter_error_definitions().into_iter().map(|inner| {
                Box::new(error::KotlinError::new(inner, ci, self.internalize)) as Box<dyn CodeDeclaration>
            }),
        )
        .chain(
            ci.iter_callback_interface_definitions()
                .into_iter()
                .map(|inner| {
                    Box::new(callback_interface::KotlinCallbackInterface::new(inner, ci, self.internalize))
                        as Box<dyn CodeDeclaration>
                }),
        )
        .chain(ci.iter_custom_types().into_iter().map(|(name, type_)| {
            let config = self.config.custom_types.get(&name).cloned();
            Box::new(custom::KotlinCustomType::new(name, type_, config, self.internalize)) as Box<dyn CodeDeclaration>
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

#[derive(Clone)]
pub struct KotlinCodeOracle {
    config: Config,
}

impl KotlinCodeOracle {
    fn new(config: Config) -> Self {
        Self { config }
    }

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
            Type::CallbackInterface(id) => {
                Box::new(callback_interface::CallbackInterfaceCodeType::new(id))
            }

            Type::Optional(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::OptionalCodeType::new(inner, outer, self.config.internalize()))
            }
            Type::Sequence(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::SequenceCodeType::new(inner, outer, self.config.internalize()))
            }
            Type::Map(ref inner) => {
                let outer = type_.clone();
                let inner = *inner.to_owned();
                Box::new(compounds::MapCodeType::new(inner, outer, self.config.internalize()))
            }
            Type::External { crate_name, name } => {
                let package_name = match self.config.external_packages.get(&crate_name) {
                    Some(name) => name.clone(),
                    None => crate_name,
                };
                Box::new(external::ExternalCodeType::new(package_name, name))
            }
            Type::Custom { name, builtin } => Box::new(custom::CustomCodeType::new(
                name.clone(),
                builtin.as_ref().clone(),
                self.config.custom_types.get(&name).cloned(),
            )),
        }
    }
}

impl CodeOracle for KotlinCodeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Box<dyn CodeType> {
        self.create_code_type(type_.clone())
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &str) -> String {
        nm.to_string().to_camel_case()
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    fn fn_name(&self, nm: &str) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    fn var_name(&self, nm: &str) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &str) -> String {
        nm.to_string().to_shouty_snake_case()
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn error_name(&self, nm: &str) -> String {
        let name = nm.to_string();
        match name.strip_suffix("Error") {
            None => name,
            Some(stripped) => {
                let mut kt_exc_name = stripped.to_owned();
                kt_exc_name.push_str("Exception");
                kt_exc_name
            }
        }
    }

    fn ffi_type_label(&self, ffi_type: &FFIType) -> String {
        match ffi_type {
            // Note that unsigned integers in Kotlin are currently experimental, but java.nio.ByteBuffer does not
            // support them yet. Thus, we use the signed variants to represent both signed and unsigned
            // types from the component API.
            FFIType::Int8 | FFIType::UInt8 => "Byte".to_string(),
            FFIType::Int16 | FFIType::UInt16 => "Short".to_string(),
            FFIType::Int32 | FFIType::UInt32 => "Int".to_string(),
            FFIType::Int64 | FFIType::UInt64 => "Long".to_string(),
            FFIType::Float32 => "Float".to_string(),
            FFIType::Float64 => "Double".to_string(),
            FFIType::RustArcPtr => "Pointer".to_string(),
            FFIType::RustBuffer => "RustBuffer.ByValue".to_string(),
            FFIType::ForeignBytes => "ForeignBytes.ByValue".to_string(),
            FFIType::ForeignCallback => "ForeignCallback".to_string(),
        }
    }
}

pub mod filters {
    use super::*;

    // This code is a bit unfortunate.  We want to have a `KotlinCodeOracle` instance available for
    // the filter functions, so that we don't always need to pass as an argument in the template
    // code.  However, `KotlinCodeOracle` depends on a `Config` instance.  So we use some dirty,
    // non-threadsafe, code to set it at the start of `generate_kotlin_bindings()`.
    //
    // If askama supported using a struct instead of a module for the filters we could avoid this.

    static mut ORACLE: Option<KotlinCodeOracle> = None;

    pub(super) fn set_oracle(oracle: KotlinCodeOracle) {
        unsafe {
            ORACLE = Some(oracle);
        }
    }

    fn oracle() -> &'static KotlinCodeOracle {
        unsafe { ORACLE.as_ref().unwrap() }
    }

    pub fn type_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(codetype.type_label(oracle()))
    }

    pub fn canonical_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(codetype.canonical_name(oracle()))
    }

    pub fn ffi_converter_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(codetype.ffi_converter_name(oracle()))
    }

    pub fn lower_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!("{}.lower", codetype.ffi_converter_name(oracle())))
    }

    pub fn allocation_size_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!(
            "{}.allocationSize",
            codetype.ffi_converter_name(oracle())
        ))
    }

    pub fn write_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!("{}.write", codetype.ffi_converter_name(oracle())))
    }

    pub fn lift_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!("{}.lift", codetype.ffi_converter_name(oracle())))
    }

    pub fn read_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!("{}.read", codetype.ffi_converter_name(oracle())))
    }

    pub fn render_literal(
        literal: &Literal,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        Ok(codetype.literal(oracle(), literal))
    }

    /// Get the Kotlin syntax for representing a given low-level `FFIType`.
    pub fn ffi_type_name(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(oracle().ffi_type_label(type_))
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().fn_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().var_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().enum_variant_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().error_name(nm))
    }
}
