/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{anyhow, bail, Result};
use askama::Template;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::{BTreeSet, HashMap};

use crate::interface::{Enum, *};

const RESERVED_WORDS: &[&str] = &[
    "alias", "and", "BEGIN", "begin", "break", "case", "class", "def", "defined?", "do", "else",
    "elsif", "END", "end", "ensure", "false", "for", "if", "module", "next", "nil", "not", "or",
    "redo", "rescue", "retry", "return", "self", "super", "then", "true", "undef", "unless",
    "until", "when", "while", "yield", "__FILE__", "__LINE__",
];

// Info for an external crate, used by `wrapper.rb` to emit `require`.
// Ruby module names for mixin *calls* are computed by `mixin_owner_module`,
// not stored here.
#[derive(Debug)]
pub struct ExternalMixin {
    pub require_path: String,
}

fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

/// A single Ruby constant identifier: `UniffiOne`, not `foo`, `Foo::Bar`, or `END`.
fn is_valid_ruby_constant(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => chars.all(|c| c.is_ascii_alphanumeric() || c == '_'),
        _ => false,
    }
}

/// Extract the crate name from a module path (everything before the first `::`).
///
/// Hyphens are normalized to underscores so UDL `[External="my-crate"]` and
/// proc-macro `module_path!()` (`my_crate`) resolve to the same key. Matches
/// [`crate::interface::ComponentInterface::namespace_for_module_path`].
fn crate_name_from_module_path(module_path: &str) -> String {
    module_path
        .split("::")
        .next()
        .unwrap_or(module_path)
        .replace('-', "_")
}

fn peel_boxes(type_: &Type) -> &Type {
    match type_ {
        Type::Box { inner_type } => peel_boxes(inner_type),
        other => other,
    }
}

fn peel_boxes_and_custom(type_: &Type) -> &Type {
    match peel_boxes(type_) {
        Type::Custom { builtin, .. } => peel_boxes_and_custom(builtin),
        other => other,
    }
}

fn askama_err(err: impl Into<anyhow::Error>) -> askama::Error {
    askama::Error::Custom(err.into().into())
}

/// Unique-within-this-component name used in generated helper identifiers
/// (`write_TypeFoo`, `alloc_from_string`, …).
///
/// Named types share the `Type` prefix so a record named `SequenceRecord` cannot
/// collide with `sequence<Record>`. Callbacks keep a distinct prefix.
pub fn canonical_name(t: &Type) -> String {
    match t {
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
        Type::Timestamp => "Timestamp".into(),
        Type::Duration => "Duration".into(),
        Type::Object { name, .. }
        | Type::Enum { name, .. }
        | Type::Record { name, .. }
        | Type::Custom { name, .. } => format!("Type{name}"),
        Type::CallbackInterface { name, .. } => format!("CallbackInterface{name}"),
        Type::Optional { inner_type } => format!("Optional{}", canonical_name(inner_type)),
        Type::Sequence { inner_type } => format!("Sequence{}", canonical_name(inner_type)),
        Type::Set { inner_type } => format!("Set{}", canonical_name(inner_type)),
        Type::Map {
            key_type,
            value_type,
        } => format!(
            "Map{}{}",
            canonical_name(key_type).to_upper_camel_case(),
            canonical_name(value_type).to_upper_camel_case()
        ),
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
    fn conversion<'a>(&'a self, primary: &'a str, fallback: &'a str) -> &'a str {
        if primary.is_empty() {
            fallback
        } else {
            primary
        }
    }

    /// Produce a Ruby expression that lifts a raw-builtin value `nm` into the custom type.
    fn lift(&self, name: &str) -> String {
        self.conversion(&self.lift, &self.into_custom)
            .replace("{}", name)
    }

    /// Produce a Ruby expression that lowers a value `nm` to its raw builtin.
    fn lower(&self, name: &str) -> String {
        self.conversion(&self.lower, &self.from_custom)
            .replace("{}", name)
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
    /// Ruby `module` name emitted in generated bindings.
    ///
    /// Default (when unset): UpperCamelCase of the UniFFI namespace.
    #[serde(default)]
    pub(super) module_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    pub(super) exclude: Vec<String>,
    #[serde(default)]
    pub(super) rename: toml::Table,
    #[serde(default)]
    pub(super) external_packages: HashMap<String, String>,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        self.cdylib_name.as_deref().unwrap_or("uniffi").to_owned()
    }

    pub fn custom_cdylib_path(&self) -> bool {
        self.cdylib_path.is_some()
    }

    pub fn cdylib_path(&self) -> String {
        self.cdylib_path.clone().unwrap_or_default()
    }

    /// Ruby module name for this crate: config override or UpperCamelCase(`namespace`).
    pub fn module_name(&self, namespace: &str) -> String {
        self.module_name
            .clone()
            .unwrap_or_else(|| class_name_rb_inner(namespace))
    }

    /// Default `module_name` when the TOML omits it: UpperCamelCase of the namespace.
    pub(super) fn default_module_name(namespace: &str) -> String {
        class_name_rb_inner(namespace)
    }

    /// Reject nested paths, reserved words, and non-constant identifiers.
    pub(super) fn validate_module_name(&self) -> Result<()> {
        let Some(name) = &self.module_name else {
            return Ok(());
        };
        if !is_valid_ruby_constant(name) || is_reserved_word(name) {
            bail!(
                "invalid [bindings.ruby.module_name] `{name}`: must be a single Ruby constant \
                 (e.g. `UniffiOne`), not nested (`Foo::Bar`) or a reserved word"
            );
        }
        Ok(())
    }

    pub fn external_package_name(&self, module_path: &str, namespace: Option<&str>) -> String {
        let crate_name = crate_name_from_module_path(module_path);

        self.external_packages
            .get(&crate_name)
            .cloned()
            .unwrap_or_else(|| class_name_rb_inner(namespace.unwrap_or(module_path)))
    }

    /// Canonicalize `external_packages` keys to underscored Rust crate names.
    ///
    /// Cargo package names and UDL `[External="..."]` values may contain hyphens;
    /// proc-macro module paths and `ci.crate_name()` never do. Rewrite keys so
    /// `my-crate` and `my_crate` are the same entry. Conflicting values for the
    /// same crate are an error.
    pub(super) fn normalize_external_package_keys(&mut self) -> Result<()> {
        let mut normalized = HashMap::with_capacity(self.external_packages.len());
        for (key, value) in &self.external_packages {
            let canon = crate_name_from_module_path(key);
            if let Some(existing) = normalized.get(&canon) {
                if existing != value {
                    bail!(
                        "conflicting [bindings.ruby.external_packages] entries for crate `{canon}`: \
                         `{key}` = `{value}` vs existing `{existing}`"
                    );
                }
            } else {
                normalized.insert(canon, value.clone());
            }
        }
        self.external_packages = normalized;
        Ok(())
    }
}

#[derive(Template)]
#[template(syntax = "rb", escape = "none", path = "wrapper.rb")]
pub struct RubyWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}

/// Owner-module policies (not interchangeable):
///
/// - mixin read/write: defining crate for named types, else this crate
/// - object/callback lift/lower (`ffi_module_prefix`): defining crate, else local
/// - record/enum/object class names in defaults (`type_class_module`)
/// - custom `uniffi_{lift,lower}_*` (`custom_owner_module`)
///
/// `ffi_module_prefix` is `None` for records/enums so alloc stays on this crate's
/// `RustBuffer`. Mixin calls are the opposite: bytes interpretation lives in the
/// defining crate.
impl<'a> RubyWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }

    pub fn module_name(&self) -> String {
        self.config.module_name(self.ci.namespace())
    }

    pub fn external_type_module(&self, module_path: &str) -> String {
        let namespace = self.ci.namespace_for_module_path(module_path).ok();
        self.config.external_package_name(module_path, namespace)
    }

    pub fn is_external_module(&self, module_path: &str) -> bool {
        crate_name_from_module_path(module_path) != self.ci.crate_name()
    }

    fn this_module_rooted(&self) -> String {
        format!("::{}", self.module_name())
    }

    fn owner_module_for_path(&self, module_path: &str) -> String {
        if self.is_external_module(module_path) {
            format!("::{}", self.external_type_module(module_path))
        } else {
            self.this_module_rooted()
        }
    }

    fn mixin_owner_module(&self, type_: &Type) -> String {
        match peel_boxes(type_).module_path() {
            Some(mp) => self.owner_module_for_path(mp),
            None => self.this_module_rooted(),
        }
    }

    fn rust_buffer_op(&self, type_: &Type, mixin: &str, op: &str) -> String {
        format!(
            "{}::{mixin}.{op}_{}",
            self.mixin_owner_module(type_),
            canonical_name(type_),
        )
    }

    /// Callee only, e.g. `::NsA::RustBufferBuilderMixin.write_TypeFoo`.
    pub fn rust_buffer_write(&self, type_: &Type) -> Result<String, askama::Error> {
        Ok(self.rust_buffer_op(type_, "RustBufferBuilderMixin", "write"))
    }

    /// Callee only, e.g. `::NsA::RustBufferStreamMixin.read_TypeFoo`.
    pub fn rust_buffer_read(&self, type_: &Type) -> Result<String, askama::Error> {
        Ok(self.rust_buffer_op(type_, "RustBufferStreamMixin", "read"))
    }

    /// `Method` object for a function's error reader, or `None`.
    ///
    /// Unwraps Custom types to find the inner Enum/Object. Named
    /// `error_reader_method_expr` so it does not collide with the Askama macro
    /// `error_reader_expr` in `macros.rb`.
    pub fn error_reader_method_expr(&self, func: &impl Callable) -> Option<String> {
        let error_type = match func.throws_type() {
            Some(Type::Custom { builtin, .. }) => builtin.as_ref(),
            Some(type_) => type_,
            None => return None,
        };
        match error_type {
            Type::Enum { .. } | Type::Object { .. } => Some(format!(
                "{}::RustBufferStreamMixin.method(:read_{})",
                self.mixin_owner_module(error_type),
                canonical_name(error_type),
            )),
            _ => None,
        }
    }

    /// Direct external crates to `require`, unique by crate.
    ///
    /// Transitive crates are not listed: nested C types are absent from
    /// `iter_external_types()`, and the intermediate crate's mixin body names C
    /// lexically (its `wrapper.rb` already `require`s C).
    pub fn external_mixin_modules(&self) -> Result<Vec<ExternalMixin>, askama::Error> {
        let mut seen_crates = BTreeSet::new();
        let mut module_to_crate = HashMap::new();
        let mut result = Vec::new();

        for typ in self.ci.iter_external_types() {
            let Some(module_path) = typ.module_path() else {
                continue;
            };
            let crate_name = crate_name_from_module_path(module_path);
            if !seen_crates.insert(crate_name.clone()) {
                continue;
            }

            // Single-UDL generation has no crate→namespace map for dependencies.
            let require_path = match self.ci.namespace_for_module_path(module_path) {
                Ok(ns) => ns.to_owned(),
                Err(_) => {
                    return Err(askama_err(anyhow!(
                        "Cannot resolve namespace for external crate `{crate_name}`. \
                         Single-UDL generation is not supported for external types; generate from a \
                         compiled library (e.g. `uniffi-bindgen generate path/to/libfoo.dylib --language ruby`) \
                         so UniFFI can load scaffolding metadata for all crates."
                    )));
                }
            };

            let module_name = self.external_type_module(module_path);
            if let Some(existing) = module_to_crate.get(&module_name) {
                return Err(askama_err(anyhow!(
                    "Ruby module `{module_name}` is used by both crate `{existing}` and crate `{crate_name}`; \
                     each crate must have a unique Ruby module name"
                )));
            }
            module_to_crate.insert(module_name, crate_name);

            result.push(ExternalMixin { require_path });
        }

        Ok(result)
    }

    /// Deduplicated require paths declared by external custom type configs.
    pub fn external_custom_type_imports(&self) -> Vec<String> {
        let mut imports = BTreeSet::new();
        for typ in self.ci.iter_external_types() {
            let Type::Custom { name, .. } = typ else {
                continue;
            };
            if let Some(extra) = self
                .config
                .custom_types
                .get(name)
                .and_then(|cfg| cfg.imports.as_ref())
            {
                imports.extend(extra.iter().cloned());
            }
        }
        imports.into_iter().collect()
    }

    /// Prefix for object/callback handle lift/lower. `None` for RustBuffer-backed
    /// types so alloc/reserve/free stay on this crate's cdylib.
    fn ffi_module_prefix(&self, type_: &Type) -> Option<String> {
        match peel_boxes_and_custom(type_) {
            Type::Object { module_path, .. } | Type::CallbackInterface { module_path, .. }
                if self.is_external_module(module_path) =>
            {
                Some(self.external_type_module(module_path))
            }
            _ => None,
        }
    }

    pub(crate) fn is_external_custom(&self, type_: &Type) -> bool {
        matches!(
            peel_boxes(type_),
            Type::Custom { module_path, .. } if self.is_external_module(module_path)
        )
    }

    pub(crate) fn custom_owner_module(&self, module_path: &str) -> String {
        self.owner_module_for_path(module_path)
    }

    pub fn lift_rb(&self, nm: &str, type_: &Type) -> Result<String, askama::Error> {
        let module = self.ffi_module_prefix(type_);
        filters::lift_rb_inner_dispatch(nm, type_, module.as_deref(), self)
    }

    pub fn lower_rb(&self, nm: impl AsRef<str>, type_: &Type) -> Result<String, askama::Error> {
        let module = self.ffi_module_prefix(type_);
        filters::lower_rb_inner_dispatch(nm.as_ref(), type_, module.as_deref(), self)
    }

    pub fn check_lower_rb(
        &self,
        nm: impl AsRef<str>,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let module = self.ffi_module_prefix(type_);
        filters::check_lower_rb_inner(nm.as_ref(), type_, module.as_deref(), self)
    }

    pub fn coerce_rb(&self, nm: impl AsRef<str>, type_: &Type) -> Result<String, askama::Error> {
        let ns = self.module_name();
        filters::coerce_rb_inner(nm, ns, type_, self)
    }

    pub fn field_default_rb(&self, field: &Field) -> Result<String, askama::Error> {
        self.default_rb(field.default_value(), &field.as_type(), "field")
    }

    pub fn arg_default_rb(&self, arg: &Argument) -> Result<String, askama::Error> {
        self.default_rb(arg.default_value(), &arg.as_type(), "arg")
    }

    fn default_rb(
        &self,
        default: Option<&DefaultValue>,
        ty: &Type,
        what: &str,
    ) -> Result<String, askama::Error> {
        match default {
            Some(default) => filters::default_rb_inner(default, ty, self),
            None => Err(askama_err(anyhow!(
                "{what} default requested but none is set"
            ))),
        }
    }

    /// Prefix for constructing a Ruby class (defaults). Unlike `ffi_module_prefix`,
    /// this includes records and enums.
    pub(crate) fn type_class_module(&self, type_: &Type) -> Option<String> {
        match peel_boxes_and_custom(type_) {
            Type::Record { module_path, .. }
            | Type::Object { module_path, .. }
            | Type::Enum { module_path, .. }
                if self.is_external_module(module_path) =>
            {
                Some(self.external_type_module(module_path))
            }
            _ => None,
        }
    }
}

fn class_name_rb_inner(nm: &str) -> String {
    nm.to_string().to_upper_camel_case()
}

mod filters {
    use super::*;

    /// Qualify `name` with an optional external module path.
    ///
    /// `qualify("Foo", Some("Mod"))` → `"::Mod::Foo"`; `qualify("", Some("Mod"))`
    /// → `"::Mod::"`; `None` leaves `name` unchanged (local, relative).
    pub(super) fn qualify(name: &str, module: Option<&str>) -> String {
        match module {
            Some(m) => format!("::{m}::{name}"),
            None => name.to_string(),
        }
    }

    /// How a type crosses the FFI after peeling `Type::Box`. Custom is *not*
    /// peeled: lift/lower wrap with `uniffi_{lift,lower}_*` then recurse.
    enum FfiShape<'a> {
        Custom {
            name: &'a str,
            builtin: &'a Type,
            module_path: &'a str,
        },
        Int,
        Float,
        Boolean,
        Object {
            name: &'a str,
        },
        Callback {
            name: &'a str,
        },
        RustBuffer,
    }

    fn ffi_shape(type_: &Type) -> FfiShape<'_> {
        match peel_boxes(type_) {
            Type::Custom {
                name,
                builtin,
                module_path,
                ..
            } => FfiShape::Custom {
                name,
                builtin,
                module_path,
            },
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64 => FfiShape::Int,
            Type::Float32 | Type::Float64 => FfiShape::Float,
            Type::Boolean => FfiShape::Boolean,
            Type::Object { name, .. } => FfiShape::Object { name },
            Type::CallbackInterface { name, .. } => FfiShape::Callback { name },
            Type::Enum { .. }
            | Type::Record { .. }
            | Type::Optional { .. }
            | Type::Sequence { .. }
            | Type::Set { .. }
            | Type::Timestamp
            | Type::String
            | Type::Bytes
            | Type::Duration
            | Type::Map { .. } => FfiShape::RustBuffer,
            Type::Box { .. } => unreachable!("peeled"),
        }
    }

    fn int_coerce_bounds(type_: &Type) -> Option<(&'static str, &'static str, &'static str)> {
        match type_ {
            Type::Int8 => Some(("i8", "-2**7", "2**7")),
            Type::Int16 => Some(("i16", "-2**15", "2**15")),
            Type::Int32 => Some(("i32", "-2**31", "2**31")),
            Type::Int64 => Some(("i64", "-2**63", "2**63")),
            Type::UInt8 => Some(("u8", "0", "2**8")),
            Type::UInt16 => Some(("u16", "0", "2**16")),
            Type::UInt32 => Some(("u32", "0", "2**32")),
            Type::UInt64 => Some(("u64", "0", "2**64")),
            _ => None,
        }
    }

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
            FfiType::Callback(name) => format!(":{name}"),
            FfiType::Reference(inner) | FfiType::MutReference(inner) => match inner.as_ref() {
                FfiType::Struct(name) => format!("{name}.by_ref"),
                _ => ":pointer".to_string(),
            },
            FfiType::VoidPointer => ":pointer".to_string(),
            FfiType::Struct(name) => format!("{name}.by_value"),
        })
    }

    /// Ruby FFI::Pointer write method for a lowered callback return.
    /// `rustbuffer` is a sentinel the template handles specially.
    #[askama::filter_fn]
    pub fn ffi_write_return_rb(
        return_type: &Type,
        _: &dyn askama::Values,
    ) -> Result<String, askama::Error> {
        let ffi_type = FfiType::from(return_type);
        match &ffi_type {
            FfiType::Int8 => Ok("write_int8".to_string()),
            FfiType::UInt8 => Ok("write_uint8".to_string()),
            FfiType::Int16 => Ok("write_int16".to_string()),
            FfiType::UInt16 => Ok("write_uint16".to_string()),
            FfiType::Int32 => Ok("write_int32".to_string()),
            FfiType::UInt32 => Ok("write_uint32".to_string()),
            FfiType::Int64 => Ok("write_int64".to_string()),
            FfiType::UInt64 => Ok("write_uint64".to_string()),
            FfiType::Float32 => Ok("write_float".to_string()),
            FfiType::Float64 => Ok("write_double".to_string()),
            FfiType::Handle => Ok("write_uint64".to_string()),
            FfiType::RustBuffer(_) => Ok("rustbuffer".to_string()),
            _ => Err(askama_err(anyhow!(
                "Unsupported FFI return type for callback: {ffi_type:?}"
            ))),
        }
    }

    /// Ruby default value for an FFI return type (async error callbacks).
    #[askama::filter_fn]
    pub fn ffi_default_value_rb(
        return_type: &Type,
        _: &dyn askama::Values,
    ) -> Result<String, askama::Error> {
        let ffi_type = FfiType::from(return_type);
        match &ffi_type {
            FfiType::Int8
            | FfiType::UInt8
            | FfiType::Int16
            | FfiType::UInt16
            | FfiType::Int32
            | FfiType::UInt32
            | FfiType::Int64
            | FfiType::UInt64
            | FfiType::Handle => Ok("0".to_string()),
            FfiType::Float32 | FfiType::Float64 => Ok("0.0".to_string()),
            FfiType::RustBuffer(_) => Ok("RustBuffer.new".to_string()),
            _ => Err(askama_err(anyhow!(
                "Unsupported FFI return type for callback: {ffi_type:?}"
            ))),
        }
    }

    /// ForeignFutureResult struct name for a method's return type.
    #[askama::filter_fn]
    pub fn foreign_future_result_rb(
        method: &Method,
        _: &dyn askama::Values,
    ) -> Result<String, askama::Error> {
        Ok(method.foreign_future_ffi_result_struct().name().to_string())
    }

    pub(super) fn default_rb_inner(
        default: &DefaultValue,
        ty: &Type,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
        match default {
            DefaultValue::Literal(lit) => literal_rb_inner(lit, ty, wrapper),
            DefaultValue::Default => type_zero_value_rb(ty, wrapper),
        }
    }

    fn literal_rb_inner(
        literal: &Literal,
        ty: &Type,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
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
            Literal::Some { inner } => {
                let inner_ty = match ty {
                    Type::Optional { inner_type } => inner_type.as_ref(),
                    // Peel Custom wrappers — the metadata construction already validated
                    // that the builtin is Optional; match type_zero_value_rb's convention.
                    Type::Custom { builtin, .. } => match builtin.as_ref() {
                        Type::Optional { inner_type } => inner_type.as_ref(),
                        other => {
                            return Err(askama_err(anyhow!(
                                "Expected Optional type for Some literal, got {other:?}"
                            )));
                        }
                    },
                    _ => {
                        return Err(askama_err(anyhow!(
                            "Expected Optional type for Some literal, got {ty:?}"
                        )));
                    }
                };
                default_rb_inner(inner, inner_ty, wrapper)?
            }
            Literal::EmptySequence => "[]".into(),
            Literal::EmptyMap => "{}".into(),
            Literal::EmptySet => "Set.new".into(),
            Literal::Enum(v, type_) => match type_ {
                Type::Enum { name, .. } => {
                    format!(
                        "{}::{}",
                        qualify(
                            &class_name_rb_inner(name),
                            wrapper.type_class_module(type_).as_deref()
                        ),
                        enum_name_rb_inner(v)?
                    )
                }
                _ => {
                    return Err(askama_err(anyhow!(
                        "Unexpected type in enum literal: {type_:?}"
                    )));
                }
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

    fn type_zero_value_rb(ty: &Type, wrapper: &RubyWrapper<'_>) -> Result<String, askama::Error> {
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
            Type::Set { .. } => "Set.new".to_string(),
            Type::Record { name, .. } | Type::Object { name, .. } => {
                format!(
                    "{}.new",
                    qualify(
                        &class_name_rb_inner(name),
                        wrapper.type_class_module(ty).as_deref()
                    )
                )
            }
            Type::Custom { builtin, .. } => type_zero_value_rb(builtin, wrapper)?,
            _ => {
                return Err(askama_err(anyhow!("No zero value for type {ty:?}")));
            }
        })
    }

    #[askama::filter_fn]
    pub fn class_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        Ok(class_name_rb_inner(nm))
    }

    #[askama::filter_fn]
    pub fn fn_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_snake_case())
    }

    #[askama::filter_fn]
    pub fn var_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        let snake = nm.to_string().to_snake_case();
        let prefix = if is_reserved_word(&snake) { "_" } else { "" };

        Ok(format!("{prefix}{snake}"))
    }

    #[askama::filter_fn]
    pub fn enum_name_rb(nm: &str, _: &dyn askama::Values) -> Result<String, askama::Error> {
        enum_name_rb_inner(nm)
    }

    pub fn enum_name_rb_inner(nm: &str) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    pub fn coerce_rb_inner<S1: AsRef<str>, S2: AsRef<str>>(
        nm: S1,
        ns: S2,
        type_: &Type,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
        let nm = nm.as_ref();
        let ns = ns.as_ref();
        Ok(match type_ {
            Type::Int8
            | Type::Int16
            | Type::Int32
            | Type::Int64
            | Type::UInt8
            | Type::UInt16
            | Type::UInt32
            | Type::UInt64 => {
                let (ty, lo, hi) = int_coerce_bounds(type_).expect("int type");
                format!("::{ns}::uniffi_in_range({nm}, \"{ty}\", {lo}, {hi})")
            }
            Type::Float32
            | Type::Float64
            | Type::Object { .. }
            | Type::Enum { .. }
            | Type::Record { .. }
            | Type::Timestamp
            | Type::Duration
            | Type::CallbackInterface { .. } => nm.to_string(),
            Type::Boolean => format!("{nm} ? true : false"),
            Type::String => format!("::{ns}::uniffi_utf8({nm})"),
            Type::Bytes => format!("::{ns}::uniffi_bytes({nm})"),
            Type::Optional { inner_type: t } => {
                format!("({nm} ? {} : nil)", coerce_rb_inner(nm, ns, t, wrapper)?)
            }
            Type::Sequence { inner_type: t } => {
                let coerce_code = coerce_rb_inner("v", ns, t, wrapper)?;
                if coerce_code == "v" {
                    nm.to_string()
                } else {
                    format!("{nm}.map {{ |v| {coerce_code} }}")
                }
            }
            Type::Set { inner_type: t } => {
                let coerce_code = coerce_rb_inner("v", ns, t, wrapper)?;
                if coerce_code == "v" {
                    nm.to_string()
                } else {
                    format!("{nm}.map {{ |v| {coerce_code} }}.to_set")
                }
            }
            Type::Map {
                key_type: kt,
                value_type: vt,
            } => {
                let k_coerce_code = coerce_rb_inner("k", ns, kt, wrapper)?;
                let v_coerce_code = coerce_rb_inner("v", ns, vt, wrapper)?;

                if k_coerce_code == "k" && v_coerce_code == "v" {
                    nm.to_string()
                } else {
                    format!(
                        "{nm}.each.with_object({{}}) {{ |(k, v), res| res[{k_coerce_code}] = {v_coerce_code} }}"
                    )
                }
            }
            Type::Box { inner_type } => coerce_rb_inner(nm, ns, inner_type, wrapper)?,
            Type::Custom { name, builtin, .. } => {
                // Config-backed and imported customs skip consumer-side builtin
                // coerce. Identity newtypes coerce inside the defining crate's
                // `uniffi_lower_*` so `to_int` / `to_str` still flow to FFI.
                if wrapper.config.custom_types.contains_key(name)
                    || wrapper.is_external_custom(type_)
                {
                    nm.to_string()
                } else {
                    coerce_rb_inner(nm, ns, builtin, wrapper)?
                }
            }
        })
    }

    pub(super) fn check_lower_rb_inner(
        nm: &str,
        type_: &Type,
        module: Option<&str>,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
        Ok(match ffi_shape(type_) {
            FfiShape::Object { name } => {
                format!(
                    "({}.uniffi_check_lower {nm})",
                    qualify(&class_name_rb_inner(name), module)
                )
            }
            FfiShape::RustBuffer => match peel_boxes(type_) {
                Type::Enum { .. }
                | Type::Record { .. }
                | Type::Optional { .. }
                | Type::Sequence { .. }
                | Type::Set { .. }
                | Type::Map { .. } => format!(
                    "{}RustBuffer.check_lower_{}({nm})",
                    qualify("", module),
                    canonical_name(type_)
                ),
                _ => String::new(),
            },
            FfiShape::Custom {
                name,
                builtin,
                module_path,
            } => {
                // External types always use the defining crate's checker
                // (identity newtypes forward to the builtin there).
                // Local types with a `type_name` use this crate's checker.
                // Identity local newtypes recurse so `LocalUrl = Url` still checks `URI`.
                let has_local_type_name = wrapper
                    .config
                    .custom_types
                    .get(name)
                    .and_then(|cfg| cfg.type_name.as_ref())
                    .is_some();
                if wrapper.is_external_module(module_path) || has_local_type_name {
                    format!(
                        "{}.uniffi_check_lower_{}({nm})",
                        wrapper.custom_owner_module(module_path),
                        canonical_name(type_),
                    )
                } else {
                    check_lower_rb_inner(nm, builtin, module, wrapper)?
                }
            }
            _ => String::new(),
        })
    }

    pub fn lower_rb_inner_dispatch(
        nm: &str,
        type_: &Type,
        module: Option<&str>,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
        Ok(match ffi_shape(type_) {
            FfiShape::Custom {
                builtin,
                module_path,
                ..
            } => {
                // Convert via the owning module, then lower the builtin.
                // Forward `module` so Object/CallbackInterface stay qualified.
                // Consumer `custom_types` live in `uniffi_lower_*` (CustomTypeTemplate.rb).
                let converted = format!(
                    "{}.uniffi_lower_{}({nm})",
                    wrapper.custom_owner_module(module_path),
                    canonical_name(type_),
                );
                lower_rb_inner_dispatch(&converted, builtin, module, wrapper)?
            }
            FfiShape::Int | FfiShape::Float => nm.to_string(),
            FfiShape::Boolean => format!("({nm} ? 1 : 0)"),
            FfiShape::Object { name } => {
                format!(
                    "({}.uniffi_lower {nm})",
                    qualify(&class_name_rb_inner(name), module)
                )
            }
            FfiShape::Callback { name } => {
                format!(
                    "({}CallbackInterface{}FfiConverter.lower {})",
                    qualify("", module),
                    class_name_rb_inner(name),
                    nm
                )
            }
            FfiShape::RustBuffer => {
                format!(
                    "{}RustBuffer.alloc_from_{}({})",
                    qualify("", module),
                    canonical_name(type_),
                    nm
                )
            }
        })
    }

    pub fn lift_rb_inner_dispatch(
        nm: &str,
        type_: &Type,
        module: Option<&str>,
        wrapper: &RubyWrapper<'_>,
    ) -> Result<String, askama::Error> {
        Ok(match ffi_shape(type_) {
            FfiShape::Custom {
                builtin,
                module_path,
                ..
            } => {
                // Lift the builtin, then convert via the owning module.
                let lifted = lift_rb_inner_dispatch(nm, builtin, module, wrapper)?;
                format!(
                    "{}.uniffi_lift_{}({lifted})",
                    wrapper.custom_owner_module(module_path),
                    canonical_name(type_),
                )
            }
            FfiShape::Int => format!("{nm}.to_i"),
            FfiShape::Float => format!("{nm}.to_f"),
            FfiShape::Boolean => format!("1 == {nm}"),
            FfiShape::Object { name } => {
                format!(
                    "{}.uniffi_lift({nm})",
                    qualify(&class_name_rb_inner(name), module)
                )
            }
            FfiShape::Callback { name } => {
                format!(
                    "({}CallbackInterface{}FfiConverter.lift {nm})",
                    qualify("", module),
                    class_name_rb_inner(name)
                )
            }
            FfiShape::RustBuffer => {
                format!("{nm}.consume_into_{}", canonical_name(type_))
            }
        })
    }

    /// Render the Ruby expression that lowers the `self` value of a trait method.
    #[askama::filter_fn]
    pub fn lower_method_self_rb(
        meth: &Method,
        _: &dyn askama::Values,
        wrapper: &RubyWrapper<'filter>,
    ) -> Result<String, askama::Error> {
        let self_type = meth
            .self_type()
            .ok_or_else(|| askama_err(anyhow!("Trait method must have a self type")))?;
        wrapper.lower_rb("self", &self_type)
    }

    /// Ruby integer literal for the discriminant of the variant at `index` in enum `e`.
    #[askama::filter_fn]
    pub fn variant_discr_literal(
        e: &Enum,
        _: &dyn askama::Values,
        index: &usize,
    ) -> Result<String, askama::Error> {
        let literal = e.variant_discr(*index).map_err(askama_err)?;

        match literal {
            Literal::UInt(v, _, _) => Ok(v.to_string()),
            Literal::Int(v, _, _) => Ok(v.to_string()),
            _ => Err(askama_err(anyhow!(
                "Only integer discriminants are supported"
            ))),
        }
    }
}

#[cfg(test)]
mod tests;
