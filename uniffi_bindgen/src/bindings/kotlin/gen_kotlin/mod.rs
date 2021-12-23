/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use serde::{Deserialize, Serialize};

use crate::backend::TemplateExpression;
use crate::interface::*;
use crate::MergeWith;

mod components;

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
    filters::set_config(config.clone());

    KotlinBindings::new(config.clone(), ci)
        .render()
        .map_err(|_| anyhow::anyhow!("failed to render kotlin bindings"))
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "KotlinBindings.kt")]
struct KotlinBindings<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    code_blocks: components::CodeBlocks,
}

impl<'a> KotlinBindings<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        let code_blocks = components::render(ci, &config);
        Self {
            config,
            ci,
            code_blocks,
        }
    }
}

pub mod filters {
    use super::*;
    use std::fmt;

    // This code is a bit unfortunate.  We want to have a `Config` instance available for the
    // filter functions.  So we use some dirty, non-threadsafe, code to set it at the start of
    // `generate_kotlin_bindings()`.
    //
    // See https://github.com/djc/askama/issues/575

    static mut CONFIG: Option<Config> = None;

    pub(super) fn set_config(config: Config) {
        unsafe {
            CONFIG = Some(config);
        }
    }

    fn config() -> &'static Config {
        unsafe { CONFIG.as_ref().unwrap() }
    }

    pub fn type_name(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(match type_.as_ref() {
            Type::UInt8 => "UByte".to_string(),
            Type::Int8 => "Byte".to_string(),
            Type::UInt16 => "UShort".to_string(),
            Type::Int16 => "Short".to_string(),
            Type::UInt32 => "UInt".to_string(),
            Type::Int32 => "Int".to_string(),
            Type::UInt64 => "ULong".to_string(),
            Type::Int64 => "Long".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Double".to_string(),
            Type::Boolean => "Boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Timestamp => "java.time.Instant".to_string(),
            Type::Duration => "java.time.Duration".to_string(),
            Type::Enum(name)
            | Type::Object(name)
            | Type::Record(name)
            | Type::CallbackInterface(name) => class_name(name)?,
            Type::Error(name) => exception_name(name)?,
            Type::Optional(inner) => format!("{}?", type_name(inner)?),
            Type::Sequence(inner) => format!("List<{}>", type_name(inner)?),
            Type::Map(ref inner) => format!("Map<String, {}>", type_name(inner)?),
            Type::External { .. } => panic!("no support for external types yet"),
            Type::Custom { name, builtin } => {
                match config().custom_types.get(name) {
                    // We have a custom type config use the supplied type name from the config
                    Some(custom_type_config) => custom_type_config
                        .type_name
                        .clone()
                        .unwrap_or_else(|| name.clone()),
                    // No custom type config, use our builtin type
                    None => type_name(builtin)?,
                }
            }
        })
    }

    pub fn ffi_converter_name(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(format!(
            "FfiConverter{}",
            &class_name(&type_.as_ref().canonical_name())?
        ))
    }

    pub fn lower_fn(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(format!("{}.lower", ffi_converter_name(type_)?))
    }

    pub fn write_fn(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(format!("{}.write", ffi_converter_name(type_)?))
    }

    pub fn lift_fn(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(format!("{}.lift", ffi_converter_name(type_)?))
    }

    pub fn read_fn(type_: &impl AsRef<Type>) -> Result<String, askama::Error> {
        Ok(format!("{}.read", ffi_converter_name(type_)?))
    }

    pub fn render_literal(literal: &Literal) -> Result<String, askama::Error> {
        fn typed_number(type_: &Type, num_str: String) -> String {
            match type_ {
                // Bytes, Shorts and Ints can all be inferred from the type.
                Type::Int8 | Type::Int16 | Type::Int32 => num_str,
                Type::Int64 => format!("{}L", num_str),

                Type::UInt8 | Type::UInt16 | Type::UInt32 => format!("{}u", num_str),
                Type::UInt64 => format!("{}uL", num_str),

                Type::Float32 => format!("{}f", num_str),
                Type::Float64 => num_str,
                _ => panic!("Unexpected literal: {} is not a number", num_str),
            }
        }

        Ok(match literal {
            Literal::Null => "null".into(),
            Literal::EmptySequence => "listOf()".into(),
            Literal::EmptyMap => "mapOf()".into(),
            Literal::Enum(v, type_) => format!("{}.{}", type_name(type_)?, enum_variant(v)?),
            Literal::Boolean(v) => format!("{}", v),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Int(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            ),
            Literal::UInt(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            ),
            Literal::Float(string, type_) => typed_number(type_, string.clone()),
        })
    }

    /// Get the Kotlin syntax for representing a given low-level `FFIType`.
    pub fn ffi_type_name(ffi_type: &FFIType) -> Result<String, askama::Error> {
        Ok(match ffi_type {
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
        })
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        let name = nm.to_string();
        Ok(match name.strip_suffix("Error") {
            None => name,
            Some(stripped) => {
                let mut kt_exc_name = stripped.to_owned();
                kt_exc_name.push_str("Exception");
                kt_exc_name
            }
        })
    }
}
