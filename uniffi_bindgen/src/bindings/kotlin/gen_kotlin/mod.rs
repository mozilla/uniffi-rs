/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeSet, HashMap};
use std::fmt;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use serde::{Deserialize, Serialize};

use crate::backend::{CodeBuilder, CodeOracle, CodeType, TemplateExpression, TypeIdentifier};
use crate::interface::*;
use crate::MergeWith;

mod callback_interface;
mod compounds;
mod custom;
mod enum_;
mod error;
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
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(format!("uniffi.{}", ci.namespace())),
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
            custom_types: HashMap::new(),
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            package_name: self.package_name.merge_with(&other.package_name),
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
            custom_types: self.custom_types.merge_with(&other.custom_types),
        }
    }
}

// Generate kotlin bindings for the given ComponentInterface, as a string.
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<String> {
    let oracle = KotlinCodeOracle::new(config.clone());
    filters::set_oracle(oracle.clone());

    KotlinBindings::new(oracle, config.clone(), ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "Bindings.kt")]
pub struct KotlinBindings<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    code_blocks: BTreeSet<String>,
    import_statements: BTreeSet<String>,
    initialization_code: BTreeSet<String>,
}
impl<'a> KotlinBindings<'a> {
    pub fn new(oracle: KotlinCodeOracle, config: Config, ci: &'a ComponentInterface) -> Self {
        let mut builder = CodeBuilder::new();
        // Generate code for all known types.  This handles both primitive types as well as
        // records, objects, etc.  BTreeSet sorts the types for nicer output.
        for type_ in ci.iter_types().iter() {
            type_.build_code(&oracle, &mut builder, ci);
        }
        // Also generate code for toplevel-functions
        for func in ci.iter_function_definitions() {
            builder.add_code_block(function::KotlinFunction::new(func, ci));
        }

        Self {
            config,
            ci,
            code_blocks: builder.code_blocks,
            import_statements: builder.import_statements,
            initialization_code: builder.initialization_code,
        }
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
            Type::External { .. } => panic!("no support for external types yet"),
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
    fn class_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_camel_case()
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_shouty_snake_case()
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn error_name(&self, nm: &dyn fmt::Display) -> String {
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
    use std::fmt;

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

    pub fn lower_var(
        nm: &dyn fmt::Display,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        Ok(codetype.lower(oracle(), nm))
    }

    pub fn write_var(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        Ok(codetype.write(oracle(), nm, target))
    }

    pub fn lift_var(
        nm: &dyn fmt::Display,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        Ok(codetype.lift(oracle(), nm))
    }

    pub fn read_var(
        nm: &dyn fmt::Display,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        Ok(codetype.read(oracle(), nm))
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
    pub fn class_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().fn_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().var_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().enum_variant_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().error_name(nm))
    }
}
