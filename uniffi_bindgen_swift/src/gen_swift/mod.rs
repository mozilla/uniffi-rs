/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use askama::Template;
use heck::{CamelCase, MixedCase};
use serde::{Deserialize, Serialize};

use super::Bindings;
use uniffi_bindgen::backend::{CodeDeclaration, CodeOracle, CodeType, TemplateExpression, TypeIdentifier};
use uniffi_bindgen::interface::*;

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

/// Config options for the caller to customize the generated Swift.
///
/// Note that this can only be used to control details of the Swift *that do not affect the underlying component*,
/// since the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub cdylib_name: String,
    /// The name of the Swift module containing the high-level foreign-language bindings.
    pub module_name: String,
    /// The name of the lower-level C module containing the FFI declarations.
    pub ffi_module_name: String,
    /// The filename stem for the lower-level C module containing the FFI declarations.
    pub ffi_module_filename: String,
    /// Whether to generate a `.modulemap` file for the lower-level C module with FFI declarations.
    pub generate_module_map: bool,
    /// Whether to omit argument labels in Swift function definitions.
    pub omit_argument_labels: bool,
    #[serde(default)]
    pub custom_types: HashMap<String, CustomTypeConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CustomTypeConfig {
    imports: Option<Vec<String>>,
    type_name: Option<String>,
    into_custom: TemplateExpression,
    from_custom: TemplateExpression,
}

impl uniffi_bindgen::BindingGeneratorConfig for Config {
    fn get_entry_from_bindings_table(bindings: &toml::Value) -> Option<toml::Value> {
        bindings.get("swift").cloned()
    }

    fn get_config_defaults(ci: &ComponentInterface) -> Vec<(String, toml::Value)> {
        vec![
            ("cdylib_name".into(), format!("uniffi_{}", ci.namespace()).into()),
            ("module_name".into(), ci.namespace().into()),
            ("ffi_module_name".into(), format!("{}FFI", ci.namespace()).into()),
            ("ffi_module_filename".into(), format!("{}FFI", ci.namespace()).into()),
            ("generate_module_map".into(), true.into()),
            ("omit_argument_labels".into(), false.into()),
        ]
    }
}

impl Config {
    /// The name of the `.modulemap` file for the lower-level C module with FFI declarations.
    pub fn modulemap_filename(&self) -> String {
        format!("{}.modulemap", self.ffi_module_filename)
    }

    /// The name of the `.h` file for the lower-level C module with FFI declarations.
    pub fn header_filename(&self) -> String {
        format!("{}.h", self.ffi_module_filename)
    }
}

/// Generate UniFFI component bindings for Swift, as strings in memory.
///
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<Bindings> {
    let header = BridgingHeader::new(config, ci)
        .render()
        .map_err(|_| anyhow!("failed to render Swift bridging header"))?;
    let library = SwiftWrapper::new(SwiftCodeOracle, config.clone(), ci)
        .render()
        .map_err(|_| anyhow!("failed to render Swift library"))?;
    let modulemap = if config.generate_module_map {
        Some(
            ModuleMap::new(config, ci)
                .render()
                .map_err(|_| anyhow!("failed to render Swift modulemap"))?,
        )
    } else {
        None
    };
    Ok(Bindings {
        library,
        header,
        modulemap,
    })
}

/// Template for generating the `.h` file that defines the low-level C FFI.
///
/// This file defines only the low-level structs and functions that are exposed
/// by the compiled Rust code. It gets wrapped into a higher-level API by the
/// code from [`SwiftWrapper`].
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "BridgingHeaderTemplate.h")]
pub struct BridgingHeader<'config, 'ci> {
    _config: &'config Config,
    ci: &'ci ComponentInterface,
}

impl<'config, 'ci> BridgingHeader<'config, 'ci> {
    pub fn new(config: &'config Config, ci: &'ci ComponentInterface) -> Self {
        Self {
            _config: config,
            ci,
        }
    }
}

/// Template for generating the `.modulemap` file that exposes the low-level C FFI.
///
/// This file defines how the low-level C FFI from [`BridgingHeader`] gets exposed
/// as a Swift module that can be called by other Swift code. In our case, its only
/// job is to define the *name* of the Swift module that will contain the FFI functions
/// so that it can be imported by the higher-level code in from [`SwiftWrapper`].
#[derive(Template)]
#[template(syntax = "c", escape = "none", path = "ModuleMapTemplate.modulemap")]
pub struct ModuleMap<'config, 'ci> {
    config: &'config Config,
    _ci: &'ci ComponentInterface,
}

impl<'config, 'ci> ModuleMap<'config, 'ci> {
    pub fn new(config: &'config Config, _ci: &'ci ComponentInterface) -> Self {
        Self { config, _ci }
    }
}

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "wrapper.swift")]
pub struct SwiftWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    oracle: SwiftCodeOracle,
}
impl<'a> SwiftWrapper<'a> {
    pub fn new(oracle: SwiftCodeOracle, config: Config, ci: &'a ComponentInterface) -> Self {
        Self { oracle, config, ci }
    }

    pub fn members(&self) -> Vec<Box<dyn CodeDeclaration + 'a>> {
        let ci = self.ci;
        vec![
            Box::new(callback_interface::SwiftCallbackInterfaceRuntime::new(ci))
                as Box<dyn CodeDeclaration>,
        ]
        .into_iter()
        .chain(
            ci.iter_enum_definitions().into_iter().map(|inner| {
                Box::new(enum_::SwiftEnum::new(inner, ci)) as Box<dyn CodeDeclaration>
            }),
        )
        .chain(ci.iter_function_definitions().into_iter().map(|inner| {
            Box::new(function::SwiftFunction::new(inner, ci, self.config.clone()))
                as Box<dyn CodeDeclaration>
        }))
        .chain(ci.iter_object_definitions().into_iter().map(|inner| {
            Box::new(object::SwiftObject::new(inner, ci, self.config.clone()))
                as Box<dyn CodeDeclaration>
        }))
        .chain(
            ci.iter_record_definitions().into_iter().map(|inner| {
                Box::new(record::SwiftRecord::new(inner, ci)) as Box<dyn CodeDeclaration>
            }),
        )
        .chain(
            ci.iter_error_definitions().into_iter().map(|inner| {
                Box::new(error::SwiftError::new(inner, ci)) as Box<dyn CodeDeclaration>
            }),
        )
        .chain(
            ci.iter_callback_interface_definitions()
                .into_iter()
                .map(|inner| {
                    Box::new(callback_interface::SwiftCallbackInterface::new(
                        inner,
                        ci,
                        self.config.clone(),
                    )) as Box<dyn CodeDeclaration>
                }),
        )
        .chain(ci.iter_custom_types().into_iter().map(|(name, type_)| {
            let config = self.config.custom_types.get(&name).cloned();
            Box::new(custom::SwiftCustomType::new(name, type_, config)) as Box<dyn CodeDeclaration>
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
pub struct SwiftCodeOracle;

impl SwiftCodeOracle {
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
            Type::Custom { name, .. } => Box::new(custom::CustomCodeType::new(name)),
        }
    }
}

impl CodeOracle for SwiftCodeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Box<dyn CodeType> {
        self.create_code_type(type_.clone())
    }

    /// Get the idiomatic Swift rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &str) -> String {
        nm.to_string().to_camel_case()
    }

    /// Get the idiomatic Swift rendering of a function name.
    fn fn_name(&self, nm: &str) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Swift rendering of a variable name.
    fn var_name(&self, nm: &str) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Swift rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &str) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Swift rendering of an exception name.
    fn error_name(&self, nm: &str) -> String {
        self.class_name(nm)
    }

    fn ffi_type_label(&self, ffi_type: &FFIType) -> String {
        match ffi_type {
            FFIType::Int8 => "int8_t".into(),
            FFIType::UInt8 => "uint8_t".into(),
            FFIType::Int16 => "int16_t".into(),
            FFIType::UInt16 => "uint16_t".into(),
            FFIType::Int32 => "int32_t".into(),
            FFIType::UInt32 => "uint32_t".into(),
            FFIType::Int64 => "int64_t".into(),
            FFIType::UInt64 => "uint64_t".into(),
            FFIType::Float32 => "float".into(),
            FFIType::Float64 => "double".into(),
            FFIType::RustArcPtr => "void*_Nonnull".into(),
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::ForeignBytes => "ForeignBytes".into(),
            FFIType::ForeignCallback => "ForeignCallback  _Nonnull".to_string(),
        }
    }
}

pub mod filters {
    use super::*;

    fn oracle() -> &'static SwiftCodeOracle {
        &SwiftCodeOracle
    }

    pub fn type_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(codetype.type_label(oracle))
    }

    pub fn canonical_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(codetype.canonical_name(oracle))
    }

    pub fn ffi_converter_name(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(codetype.ffi_converter_name(oracle()))
    }

    pub fn lower_fn(codetype: &impl CodeType) -> Result<String, askama::Error> {
        Ok(format!("{}.lower", codetype.ffi_converter_name(oracle())))
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

    pub fn literal_swift(
        literal: &Literal,
        codetype: &impl CodeType,
    ) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(codetype.literal(oracle, literal))
    }

    /// Get the Swift syntax for representing a given low-level `FFIType`.
    pub fn ffi_type_name(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(oracle().ffi_type_label(type_))
    }

    /// Get the type that a type is lowered into.  This is subtly different than `type_ffi`, see
    /// #1106 for details
    pub fn type_ffi_lowered(ffi_type: &FFIType) -> Result<String, askama::Error> {
        Ok(match ffi_type {
            FFIType::Int8 => "Int8".into(),
            FFIType::UInt8 => "UInt8".into(),
            FFIType::Int16 => "Int16".into(),
            FFIType::UInt16 => "UInt16".into(),
            FFIType::Int32 => "Int32".into(),
            FFIType::UInt32 => "UInt32".into(),
            FFIType::Int64 => "Int64".into(),
            FFIType::UInt64 => "UInt64".into(),
            FFIType::Float32 => "float".into(),
            FFIType::Float64 => "double".into(),
            FFIType::RustArcPtr => "void*_Nonnull".into(),
            FFIType::RustBuffer => "RustBuffer".into(),
            FFIType::ForeignBytes => "ForeignBytes".into(),
            FFIType::ForeignCallback => "ForeignCallback  _Nonnull".to_string(),
        })
    }

    /// Get the idiomatic Swift rendering of a class name (for enums, records, errors, etc).
    pub fn class_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Swift rendering of a function name.
    pub fn fn_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().fn_name(nm))
    }

    /// Get the idiomatic Swift rendering of a variable name.
    pub fn var_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().var_name(nm))
    }

    /// Get the idiomatic Swift rendering of an individual enum variant.
    pub fn enum_variant_swift(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().enum_variant_name(nm))
    }
}
