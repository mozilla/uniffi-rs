/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::Result;
use askama::Template;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::interface::{Enum, *};

const RESERVED_WORDS: &[&str] = &[
    "alias", "and", "BEGIN", "begin", "break", "case", "class", "def", "defined?", "do", "else",
    "elsif", "END", "end", "ensure", "false", "for", "if", "module", "next", "nil", "not", "or",
    "redo", "rescue", "retry", "return", "self", "super", "then", "true", "undef", "unless",
    "until", "when", "while", "yield", "__FILE__", "__LINE__",
];

fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

/// Get the canonical, unique-within-this-component name for a type.
///
/// When generating helper code for foreign language bindings, it's sometimes useful to be
/// able to name a particular type in order to e.g. call a helper function that is specific
/// to that type. We support this by defining a naming convention where each type gets a
/// unique canonical name, constructed recursively from the names of its component types (if any).
pub fn canonical_name(t: &Type) -> String {
    match t {
        // Builtin primitive types, with plain old names.
        Type::Int8 => "i8".into(),
        Type::UInt8 => "u8".into(),
        Type::Int16 => "i16".into(),
        Type::UInt16 => "u16".into(),
        Type::Int32 => "i32".into(),
        Type::UInt32 => "u32".into(),
        Type::Int64 => "i64".into(),
        Type::UInt64 => "u64".into(),
        Type::Float32 => "f32".into(),
        Type::Float64 => "f64".into(),
        Type::String => "string".into(),
        Type::Bytes => "bytes".into(),
        Type::Boolean => "bool".into(),
        // API defined types.
        // Note that these all get unique names, and the parser ensures that the names do not
        // conflict with a builtin type. We add a prefix to the name to guard against pathological
        // cases like a record named `SequenceRecord` interfering with `sequence<Record>`.
        // However, types that support importing all end up with the same prefix of "Type", so
        // that the import handling code knows how to find the remote reference.
        Type::Object { name, .. } => format!("Type{name}"),
        Type::Enum { name, .. } => format!("Type{name}"),
        Type::Record { name, .. } => format!("Type{name}"),
        Type::CallbackInterface { name, .. } => format!("CallbackInterface{name}"),
        Type::Timestamp => "Timestamp".into(),
        Type::Duration => "Duration".into(),
        // Recursive types.
        // These add a prefix to the name of the underlying type.
        // The component API definition cannot give names to recursive types, so as long as the
        // prefixes we add here are all unique amongst themselves, then we have no chance of
        // acccidentally generating name collisions.
        Type::Optional { inner_type } => format!("Optional{}", canonical_name(inner_type)),
        Type::Sequence { inner_type } => format!("Sequence{}", canonical_name(inner_type)),
        Type::Map {
            key_type,
            value_type,
        } => format!(
            "Map{}{}",
            canonical_name(key_type).to_upper_camel_case(),
            canonical_name(value_type).to_upper_camel_case()
        ),
        Type::Custom { name, .. } => format!("Type{name}"),
        Type::Box { inner_type } => canonical_name(inner_type),
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTypeConfig {
    type_name: Option<String>,
    imports: Option<Vec<String>>,
    into_custom: String, // b/w compat alias for lift
    lift: String,
    from_custom: String, // b/w compat alias for lower
    lower: String,
}

impl CustomTypeConfig {
    /// Produce a Ruby expression that lifts a raw-builtin value `nm` into the custom type.
    fn lift(&self, name: &str) -> String {
        let converter = if self.lift.is_empty() {
            &self.into_custom
        } else {
            &self.lift
        };
        converter.replace("{}", name)
    }

    /// Produce a Ruby expression that lowers a value `nm` to its raw builtin.
    fn lower(&self, name: &str) -> String {
        let converter = if self.lower.is_empty() {
            &self.from_custom
        } else {
            &self.lower
        };
        converter.replace("{}", name)
    }

    /// True if this config actually specifies conversion expressions.
    pub fn has_conversion(&self) -> bool {
        !self.lift.is_empty() || !self.into_custom.is_empty()
    }
}

// Some config options for it the caller wants to customize the generated ruby.
// Note that this can only be used to control details of the ruby *that do not affect the underlying component*,
// since the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) cdylib_name: Option<String>,
    cdylib_path: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    pub(super) exclude: Vec<String>,
    #[serde(default)]
    pub(super) rename: toml::Table,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        self.cdylib_name
            .clone()
            .unwrap_or_else(|| "uniffi".to_string())
    }

    pub fn custom_cdylib_path(&self) -> bool {
        self.cdylib_path.is_some()
    }

    pub fn cdylib_path(&self) -> String {
        self.cdylib_path.clone().unwrap_or_default()
    }
}

#[derive(Template)]
#[template(syntax = "rb", escape = "none", path = "wrapper.rb")]
pub struct RubyWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> RubyWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }
}

mod filters {
    use super::*;

    #[askama::filter_fn]
    pub fn type_ffi(type_: &FfiType, _: &dyn askama::Values) -> Result<String, askama::Error> {
        Ok(match type_ {
            FfiType::Int8 => ":int8".to_string(),
            FfiType::UInt8 => ":uint8".to_string(),
            FfiType::Int16 => ":int16".to_string(),
            FfiType::UInt16 => ":uint16".to_string(),
            FfiType::Int32 => ":int32".to_string(),
            FfiType::UInt32 => ":uint32".to_string(),
            FfiType::Int64 => ":int64".to_string(),
            FfiType::UInt64 => ":uint64".to_string(),
            FfiType::Float32 => ":float".to_string(),
            FfiType::Float64 => ":double".to_string(),
            FfiType::Handle => ":uint64".to_string(),
            FfiType::RustBuffer(_) => "RustBuffer.by_value".to_string(),
            FfiType::RustCallStatus => "RustCallStatus".to_string(),
            FfiType::ForeignBytes => "ForeignBytes".to_string(),
            FfiType::Callback(_) => unimplemented!("FFI Callbacks not implemented"),
            // Note: this can't just be `unimplemented!()` because some of the FFI function
            // definitions use references.  Those FFI functions aren't actually used, so we just
            // pick something that runs and makes some sense.  Revisit this once the references
            // are actually implemented.
            FfiType::Reference(_) | FfiType::MutReference(_) => ":pointer".to_string(),
            FfiType::VoidPointer => ":pointer".to_string(),
            FfiType::Struct(_) => {
                unimplemented!("Structs are not implemented")
            }
        })
    }

    fn default_rb_inner(default: &DefaultValue) -> Result<String, askama::Error> {
        let DefaultValue::Literal(literal) = default else {
            unimplemented!("not supported.");
        };
        literal_rb_inner(literal)
    }

    fn literal_rb_inner(literal: &Literal) -> Result<String, askama::Error> {
        Ok(match literal {
            Literal::Boolean(v) => {
                if *v {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            // use the double-quote form to match with the other languages, and quote escapes.
            Literal::String(s) => format!("\"{s}\""),
            Literal::None => "nil".into(),
            Literal::Some { inner } => default_rb_inner(inner)?,
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "{}".into(),
            Literal::Enum(v, type_) => match type_ {
                Type::Enum { name, .. } => {
                    format!("{}::{}", class_name_rb_inner(name)?, enum_name_rb_inner(v)?)
                }
                _ => panic!("Unexpected type in enum literal: {type_:?}"),
            },
            // https://docs.ruby-lang.org/en/2.0.0/syntax/literals_rdoc.html
            Literal::Int(i, radix, _) => match radix {
                Radix::Octal => format!("0o{i:o}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
            Literal::UInt(i, radix, _) => match radix {
                Radix::Octal => format!("0o{i:o}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
            Literal::Float(string, _type_) => string.clone(),
        })
    }

    /// Return the Ruby zero/default value for a type (used for `#[uniffi::default]`).
    fn type_zero_value_rb(ty: &Type) -> Result<String, askama::Error> {
        Ok(match ty {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => "0".to_string(),
            Type::Float32 | Type::Float64 => "0.0".to_string(),
            Type::Boolean => "false".to_string(),
            Type::String => "\"\"".to_string(),
            Type::Optional { .. } => "nil".to_string(),
            Type::Sequence { .. } => "[]".to_string(),
            Type::Bytes => "\"\".b".to_string(),
            Type::Map { .. } => "{}".to_string(),
            // Named types with no-arg constructors
            Type::Record { name, .. } | Type::Object { name, .. } => {
                format!("{}.new", class_name_rb_inner(name)?)
            }
            // Custom types delegate to their underlying builtin
            Type::Custom { builtin, .. } => type_zero_value_rb(builtin)?,
            _ => {
                return Err(askama::Error::Custom(
                    anyhow::anyhow!("No zero value for type {ty:?}").into(),
                ))
            }
        })
    }

    /// Render the Ruby default value for a field, handling both `Default` and `Literal` variants.
    #[askama::filter_fn]
    pub fn field_default_rb(
        field: &Field,
        _: &dyn askama::Values,
    ) -> Result<String, askama::Error> {
        match field.default_value() {
            Some(DefaultValue::Default) => {
                let ty = field.as_type();
                type_zero_value_rb(&ty)
            }
            Some(DefaultValue::Literal(lit)) => literal_rb_inner(lit),
            None => Err(askama::Error::Custom(
                anyhow::anyhow!("field_default_rb called on field with no default value").into(),
            )),
        }
    }

    /// Render the Ruby default value for a function/method argument.
    #[askama::filter_fn]
    pub fn arg_default_rb(arg: &Argument, _: &dyn askama::Values) -> Result<String, askama::Error> {
        match arg.default_value() {
            Some(DefaultValue::Default) => type_zero_value_rb(&arg.as_type()),
            Some(DefaultValue::Literal(lit)) => literal_rb_inner(lit),
            None => Err(askama::Error::Custom(
                anyhow::anyhow!("arg_default_rb called on arg with no default value").into(),
            )),
        }
    }

    #[askama::filter_fn]
    pub fn class_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        class_name_rb_inner(nm)
    }

    fn class_name_rb_inner(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_upper_camel_case())
    }

    #[askama::filter_fn]
    pub fn fn_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    #[askama::filter_fn]
    pub fn var_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        let prefix = if is_reserved_word(&nm) { "_" } else { "" };

        Ok(format!("{prefix}{}", nm.to_snake_case()))
    }

    #[askama::filter_fn]
    pub fn enum_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        enum_name_rb_inner(nm)
    }

    pub fn enum_name_rb_inner(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    #[askama::filter_fn]
    pub fn coerce_rb<S1: AsRef<str>, S2: AsRef<str>>(
        nm: S1,
        _: &dyn askama::Values,
        ns: S2,
        type_: &Type,
        config: &Config,
    ) -> Result<String, askama::Error> {
        coerce_rb_inner(nm, ns, type_, &config.custom_types)
    }

    pub fn coerce_rb_inner<S1: AsRef<str>, S2: AsRef<str>>(
        nm: S1,
        ns: S2,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        let nm = nm.as_ref();
        let ns = ns.as_ref();
        Ok(match type_ {
            Type::Int8 => format!("{ns}::uniffi_in_range({nm}, \"i8\", -2**7, 2**7)"),
            Type::Int16 => format!("{ns}::uniffi_in_range({nm}, \"i16\", -2**15, 2**15)"),
            Type::Int32 => format!("{ns}::uniffi_in_range({nm}, \"i32\", -2**31, 2**31)"),
            Type::Int64 => format!("{ns}::uniffi_in_range({nm}, \"i64\", -2**63, 2**63)"),
            Type::UInt8 => format!("{ns}::uniffi_in_range({nm}, \"u8\", 0, 2**8)"),
            Type::UInt16 => format!("{ns}::uniffi_in_range({nm}, \"u16\", 0, 2**16)"),
            Type::UInt32 => format!("{ns}::uniffi_in_range({nm}, \"u32\", 0, 2**32)"),
            Type::UInt64 => format!("{ns}::uniffi_in_range({nm}, \"u64\", 0, 2**64)"),
            Type::Float32 | Type::Float64 => nm.to_string(),
            Type::Boolean => format!("{nm} ? true : false"),
            Type::Object { .. } | Type::Enum { .. } | Type::Record { .. } => nm.to_string(),
            Type::String => format!("{ns}::uniffi_utf8({nm})"),
            Type::Bytes => format!("{ns}::uniffi_bytes({nm})"),
            Type::Timestamp | Type::Duration => nm.to_string(),
            Type::CallbackInterface { name, .. } => {
                // Callback interfaces are not yet supported; emit a Ruby runtime error so that
                // code generation succeeds but calling such functions raises clearly.
                format!("raise NotImplementedError, \"Callback interface {name} is not yet supported in the Ruby bindings\"")
            }
            Type::Optional { inner_type: t } => {
                format!(
                    "({nm} ? {} : nil)",
                    coerce_rb_inner(nm, ns, t, custom_types)?
                )
            }
            Type::Sequence { inner_type: t } => {
                let coerce_code = coerce_rb_inner("v", ns, t, custom_types)?;
                if coerce_code == "v" {
                    nm.to_string()
                } else {
                    format!("{nm}.map {{ |v| {coerce_code} }}")
                }
            }
            Type::Map { value_type: t, .. } => {
                let k_coerce_code = coerce_rb_inner("k", ns, &Type::String, custom_types)?;
                let v_coerce_code = coerce_rb_inner("v", ns, t, custom_types)?;

                if k_coerce_code == "k" && v_coerce_code == "v" {
                    nm.to_string()
                } else {
                    format!(
                        "{nm}.each.with_object({{}}) {{ |(k, v), res| res[{k_coerce_code}] = {v_coerce_code} }}"
                    )
                }
            }
            Type::Box { inner_type } => coerce_rb_inner(nm, ns, inner_type, custom_types)?,
            Type::Custom { name, builtin, .. } => {
                // For config-backed custom types, the user passes a custom-typed values;
                // skip builtin coercion (the lower expression handles conversion).
                if custom_types.contains_key(name) {
                    nm.to_string()
                } else {
                    coerce_rb_inner(nm, ns, builtin, custom_types)?
                }
            }
        })
    }

    #[askama::filter_fn]
    pub fn check_lower_rb<S: AsRef<str>>(
        nm: S,
        _: &dyn askama::Values,
        type_: &Type,
        config: &Config,
    ) -> Result<String, askama::Error> {
        let nm = nm.as_ref();
        Ok(match type_ {
            Type::Object { name, .. } => {
                format!("({}.uniffi_check_lower {nm})", class_name_rb_inner(name)?)
            }
            Type::Enum { .. }
            | Type::Record { .. }
            | Type::Optional { .. }
            | Type::Sequence { .. }
            | Type::Map { .. } => format!(
                "RustBuffer.check_lower_{}({})",
                class_name_rb_inner(&canonical_name(type_))?,
                nm
            ),
            Type::Custom { name, .. } => {
                if let Some(cfg) = config.custom_types.get(name) {
                    if let Some(type_name) = &cfg.type_name {
                        // Emit: rause TypeError, "Expected URI, got #{nm.class}" unless nm.is_a?(URI)
                        format!(
                            "raise TypeError, \"Expected {type_name}, got {{#{nm}.class}}\" unless {nm}.is_a?({type_name})"
                        )
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            }
            _ => "".to_owned(),
        })
    }

    pub fn lower_rb_inner(
        nm: &str,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        let mut type_ = type_;
        while let Type::Box { inner_type } = type_ {
            type_ = &**inner_type;
        }
        lower_rb_inner_no_box(nm, type_, custom_types)
    }

    fn lower_rb_inner_no_box(
        nm: &str,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Box { inner_type } => lower_rb_inner(nm, inner_type, custom_types)?,
            Type::Custom { name, builtin, .. } => {
                if let Some(cfg) = custom_types.get(name) {
                    // Apply the configured `lower` expression, then lower the resulting builtin.
                    let lowered_nm = cfg.lower(nm);
                    lower_rb_inner(&lowered_nm, builtin, custom_types)?
                } else {
                    lower_rb_inner(nm, builtin, custom_types)?
                }
            }
            type_ => return lower_rb_inner_dispatch(nm, type_),
        })
    }

    pub fn lower_rb_inner_dispatch(nm: &str, type_: &Type) -> Result<String, askama::Error> {
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
            Type::Boolean => format!("({nm} ? 1 : 0)"),
            Type::String => format!("RustBuffer.allocFromString({nm})"),
            Type::Bytes => format!("RustBuffer.allocFromBytes({nm})"),
            Type::Object { name, .. } => {
                format!("({}.uniffi_lower {nm})", class_name_rb_inner(name)?)
            }
            Type::CallbackInterface { name, .. } => {
                format!(
                    "raise(NotImplementedError, \"Callback interface {name} is not yet supported in the Ruby bindings\")"
                )
            }
            Type::Enum { .. }
            | Type::Record { .. }
            | Type::Optional { .. }
            | Type::Sequence { .. }
            | Type::Timestamp
            | Type::Duration
            | Type::Map { .. } => format!(
                "RustBuffer.alloc_from_{}({})",
                class_name_rb_inner(&canonical_name(type_))?,
                nm
            ),
            Type::Box { .. } => unreachable!(),
            Type::Custom { .. } => unreachable!("Custom types should be handled before dispatch"),
        })
    }

    #[askama::filter_fn]
    pub fn lower_rb(
        nm: &str,
        _: &dyn askama::Values,
        type_: &Type,
        config: &Config,
    ) -> Result<String, askama::Error> {
        lower_rb_inner(nm, type_, &config.custom_types)
    }

    fn lift_rb_inner(
        nm: &str,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        let mut type_ = type_;
        while let Type::Box { inner_type } = type_ {
            type_ = &**inner_type;
        }
        lift_rb_inner_no_box(nm, type_, custom_types)
    }

    fn lift_rb_inner_no_box(
        nm: &str,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Box { inner_type } => lift_rb_inner(nm, inner_type, custom_types)?,
            Type::Custom { name, builtin, .. } => {
                // First lift the raw builtin value, then apply the configured lift expression.
                let lifted = lift_rb_inner(nm, builtin, custom_types)?;
                if let Some(cfg) = custom_types.get(name) {
                    cfg.lift(&lifted)
                } else {
                    lifted
                }
            }
            type_ => return lift_rb_inner_dispatch(nm, type_, custom_types),
        })
    }

    pub fn lift_rb_inner_dispatch(
        nm: &str,
        type_: &Type,
        custom_types: &HashMap<String, CustomTypeConfig>,
    ) -> Result<String, askama::Error> {
        Ok(match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => format!("{nm}.to_i"),
            Type::Float32 | Type::Float64 => format!("{nm}.to_f"),
            Type::Boolean => format!("1 == {nm}"),
            Type::String => format!("{nm}.consumeIntoString"),
            Type::Bytes => format!("{nm}.consumeIntoBytes"),
            Type::Object { name, .. } => {
                format!("{}.uniffi_allocate({nm})", class_name_rb_inner(name)?)
            }
            Type::CallbackInterface { name, .. } => {
                format!(
                    "raise(NotImplementedError, \"Callback interface {name} is not yet supported in the Ruby bindings\")"
                )
            }
            Type::Enum { .. } => {
                format!(
                    "{}.consumeInto{}",
                    nm,
                    class_name_rb_inner(&canonical_name(type_))?
                )
            }
            Type::Record { .. }
            | Type::Optional { .. }
            | Type::Sequence { .. }
            | Type::Timestamp
            | Type::Duration
            | Type::Map { .. } => format!(
                "{}.consumeInto{}",
                nm,
                class_name_rb_inner(&canonical_name(type_))?
            ),
            Type::Box { .. } => unreachable!(),
            Type::Custom { name, builtin, .. } => {
                let lifted = lift_rb_inner(nm, builtin, custom_types)?;
                if let Some(cfg) = custom_types.get(name) {
                    cfg.lift(&lifted)
                } else {
                    lifted
                }
            }
        })
    }

    #[askama::filter_fn]
    pub fn lift_rb(
        nm: &str,
        _: &dyn askama::Values,
        type_: &Type,
        config: &Config,
    ) -> Result<String, askama::Error> {
        lift_rb_inner(nm, type_, &config.custom_types)
    }

    /// Render a Ruby integer literal for the discriminant of the variant at `index` in enum `e`.
    #[askama::filter_fn]
    pub fn variant_discr_literal(
        e: &Enum,
        _: &dyn askama::Values,
        index: &usize,
    ) -> Result<String, askama::Error> {
        let literal = e
            .variant_discr(*index)
            .map_err(|err| askama::Error::Custom(err.into()))?;

        match literal {
            Literal::UInt(v, _, _) => Ok(v.to_string()),
            Literal::Int(v, _, _) => Ok(v.to_string()),
            _ => Err(askama::Error::Custom(
                anyhow::anyhow!("Only integer discriminants are supported").into(),
            )),
        }
    }
}

#[cfg(test)]
mod test_type {
    use super::*;

    #[test]
    fn test_canonical_names() {
        // Non-exhaustive, but gives a bit of a flavour of what we want.
        assert_eq!(canonical_name(&Type::UInt8), "u8");
        assert_eq!(canonical_name(&Type::String), "string");
        assert_eq!(canonical_name(&Type::Bytes), "bytes");
        assert_eq!(
            canonical_name(&Type::Optional {
                inner_type: Box::new(Type::Sequence {
                    inner_type: Box::new(Type::Object {
                        module_path: "anything".to_string(),
                        name: "Example".into(),
                        imp: ObjectImpl::Struct,
                    })
                })
            }),
            "OptionalSequenceTypeExample"
        );
    }
}

#[cfg(test)]
mod tests;
