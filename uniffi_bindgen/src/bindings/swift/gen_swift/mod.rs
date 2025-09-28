/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use once_cell::sync::Lazy;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Debug;

use anyhow::{Context, Result};
use askama::Template;

use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};

use super::Bindings;

use crate::interface::*;

mod callback_interface;
mod compounds;
mod custom;
mod enum_;
mod miscellany;
mod object;
mod primitives;
mod record;

/// A trait tor the implementation.
trait CodeType: Debug {
    /// The language specific label used to reference this type. This will be used in
    /// method signatures and property declarations.
    fn type_label(&self) -> String;

    /// A representation of this type label that can be used as part of another
    /// identifier. e.g. `read_foo()`, or `FooInternals`.
    ///
    /// This is especially useful when creating specialized objects or methods to deal
    /// with this type only.
    fn canonical_name(&self) -> String {
        self.type_label()
    }

    // default for named types is to assume a ctor exists.
    fn default(&self, default: &DefaultValue) -> Result<String> {
        match default {
            DefaultValue::Default => Ok(format!("{}()", self.type_label())),
            DefaultValue::Literal(_) => crate::bail!("Literals for named types are not supported"),
        }
    }

    /// Name of the FfiConverter
    ///
    /// This is the object that contains the lower, write, lift, and read methods for this type.
    fn ffi_converter_name(&self) -> String {
        format!("FfiConverter{}", self.canonical_name())
    }

    /// Function to run at startup
    fn initialization_fn(&self) -> Option<String> {
        None
    }
}

/// From <https://docs.swift.org/swift-book/documentation/the-swift-programming-language/lexicalstructure/#Keywords-and-Punctuation>
static KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    [
        // Keywords used in declarations:
        "associatedtype",
        "class",
        "deinit",
        "enum",
        "extension",
        "fileprivate",
        "func",
        "import",
        "init",
        "inout",
        "internal",
        "let",
        "open",
        "operator",
        "private",
        "precedencegroup",
        "protocol",
        "public",
        "rethrows",
        "static",
        "struct",
        "subscript",
        "typealias",
        "var",
        // Keywords used in statements:
        "break",
        "case",
        "catch",
        "continue",
        "default",
        "defer",
        "do",
        "else",
        "fallthrough",
        "for",
        "guard",
        "if",
        "in",
        "repeat",
        "return",
        "throw",
        "switch",
        "where",
        "while",
        // Keywords used in expressions and types:
        "Any",
        "as",
        "await",
        "catch",
        "false",
        "is",
        "nil",
        "rethrows",
        "self",
        "Self",
        "super",
        "throw",
        "throws",
        "true",
        "try",
    ]
    .iter()
    .map(ToString::to_string)
    .collect::<HashSet<_>>()
});

/// Quote a name for use in a context where keywords must be quoted
pub fn quote_general_keyword(nm: String) -> String {
    if KEYWORDS.contains(&nm) {
        format!("`{nm}`")
    } else {
        nm
    }
}

/// Per <https://docs.swift.org/swift-book/documentation/the-swift-programming-language/lexicalstructure/#Keywords-and-Punctuation> subset of keywords which need quoting in arg context.
static ARG_KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    ["inout", "var", "let"]
        .iter()
        .map(ToString::to_string)
        .collect::<HashSet<_>>()
});

/// Quote a name for use in arg context where fewer keywords must be quoted
pub fn quote_arg_keyword(nm: String) -> String {
    if ARG_KEYWORDS.contains(&nm) {
        format!("`{nm}`")
    } else {
        nm
    }
}

/// Config options for the caller to customize the generated Swift.
///
/// Note that this can only be used to control details of the Swift *that do not affect the underlying component*,
/// since the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) module_name: Option<String>,
    ffi_module_name: Option<String>,
    ffi_module_filename: Option<String>,
    generate_module_map: Option<bool>,
    #[serde(default)]
    omit_checksums: bool,
    omit_argument_labels: Option<bool>,
    generate_immutable_records: Option<bool>,
    omit_localized_error_conformance: Option<bool>,
    generate_case_iterable_conformance: Option<bool>,
    generate_codable_conformance: Option<bool>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    link_frameworks: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTypeConfig {
    imports: Option<Vec<String>>,
    type_name: Option<String>,
    into_custom: String, // b/w compat alias for lift
    lift: String,
    from_custom: String, // b/w compat alias for lower
    lower: String,
}

// functions replace literal "{}" in strings with a specified value.
impl CustomTypeConfig {
    fn lift(&self, name: &str) -> String {
        let converter = if self.lift.is_empty() {
            &self.into_custom
        } else {
            &self.lift
        };
        converter.replace("{}", name)
    }
    fn lower(&self, name: &str) -> String {
        let converter = if self.lower.is_empty() {
            &self.from_custom
        } else {
            &self.lower
        };
        converter.replace("{}", name)
    }
}

impl Config {
    /// The name of the Swift module containing the high-level foreign-language bindings.
    /// Panics if the module name hasn't been configured.
    pub fn module_name(&self) -> String {
        self.module_name
            .as_ref()
            .expect("module name should have been set in update_component_configs")
            .clone()
    }

    /// The name of the lower-level C module containing the FFI declarations.
    pub fn ffi_module_name(&self) -> String {
        match self.ffi_module_name.as_ref() {
            Some(name) => name.clone(),
            None => format!("{}FFI", self.module_name()),
        }
    }

    /// The filename stem for the lower-level C module containing the FFI declarations.
    pub fn ffi_module_filename(&self) -> String {
        match self.ffi_module_filename.as_ref() {
            Some(name) => name.clone(),
            None => self.ffi_module_name(),
        }
    }

    /// The name of the `.modulemap` file for the lower-level C module with FFI declarations.
    pub fn modulemap_filename(&self) -> String {
        format!("{}.modulemap", self.ffi_module_filename())
    }

    /// The name of the `.h` file for the lower-level C module with FFI declarations.
    pub fn header_filename(&self) -> String {
        format!("{}.h", self.ffi_module_filename())
    }

    /// Whether to generate a `.modulemap` file for the lower-level C module with FFI declarations.
    pub fn generate_module_map(&self) -> bool {
        self.generate_module_map.unwrap_or(true)
    }

    /// Whether to omit argument labels in Swift function definitions.
    pub fn omit_argument_labels(&self) -> bool {
        self.omit_argument_labels.unwrap_or(false)
    }

    /// Whether to generate immutable records (`let` instead of `var`)
    pub fn generate_immutable_records(&self) -> bool {
        self.generate_immutable_records.unwrap_or(false)
    }

    /// Whether to make generated error types conform to `LocalizedError`. Default: false.
    pub fn omit_localized_error_conformance(&self) -> bool {
        self.omit_localized_error_conformance.unwrap_or(false)
    }

    /// Whether to make simple generated enum and error types conform to `CaseIterable`. Default: false.
    pub fn generate_case_iterable_conformance(&self) -> bool {
        self.generate_case_iterable_conformance.unwrap_or(false)
    }

    /// Whether to make generated records, enums and errors conform to `Codable`. Default: false.
    pub fn generate_codable_conformance(&self) -> bool {
        self.generate_codable_conformance.unwrap_or(false)
    }

    /// Extra frameworks to link this Swift module against. This is populated in the modulemap file,
    /// usually as part of an `xcframework`.
    pub fn link_frameworks(&self) -> Vec<String> {
        self.link_frameworks.clone()
    }

    /// Programmatically generate the conformances for Record
    pub fn conformance_list_for_record(
        &self,
        rec: &Record,
        contains_object_references: &bool,
    ) -> String {
        let mut conformances = vec!["Sendable"];

        let uniffi_trait_methods = rec.uniffi_trait_methods();

        if !contains_object_references || uniffi_trait_methods.eq_eq.is_some() {
            conformances.push("Equatable");
        }

        if !contains_object_references || uniffi_trait_methods.hash_hash.is_some() {
            conformances.push("Hashable");
        }

        if uniffi_trait_methods.ord_cmp.is_some() {
            conformances.push("Comparable");
        }

        if uniffi_trait_methods.debug_fmt.is_some() {
            conformances.push("CustomDebugStringConvertible");
        }

        if uniffi_trait_methods.display_fmt.is_some() {
            conformances.push("CustomStringConvertible");
        }

        // Objects can't be Codable at the moment, so we can't derive `Codable` conformance if this Record references one
        if !contains_object_references && self.generate_codable_conformance() {
            conformances.push("Codable");
        }

        conformances.join(", ")
    }

    /// Programmatically generate the conformances for an Enum
    pub fn conformance_list_for_enum(&self, e: &Enum, contains_object_references: &bool) -> String {
        let uniffi_trait_methods = e.uniffi_trait_methods();

        let mut conformances = vec!["Sendable"];

        // We auto-generate `Equatable, Hashable`, but only if we have no objects. We could do better - see #2409
        if !contains_object_references || uniffi_trait_methods.eq_eq.is_some() {
            conformances.push("Equatable");
        }

        if !contains_object_references || uniffi_trait_methods.hash_hash.is_some() {
            conformances.push("Hashable");
        }

        if uniffi_trait_methods.ord_cmp.is_some() {
            conformances.push("Comparable");
        }

        if uniffi_trait_methods.debug_fmt.is_some() {
            conformances.push("CustomDebugStringConvertible");
        }

        if uniffi_trait_methods.display_fmt.is_some() {
            conformances.push("CustomStringConvertible");
        }

        // Objects can't be Codable at the moment, so we can't derive `Codable` conformance if this Enum references one
        if !contains_object_references && self.generate_codable_conformance() {
            conformances.push("Codable");
        }

        if self.generate_case_iterable_conformance() && e.is_flat() && !e.contains_variant_fields()
        {
            conformances.push("CaseIterable");
        }

        conformances.join(", ")
    }

    pub fn conformance_list_for_error(
        &self,
        e: &Enum,
        contains_object_references: &bool,
    ) -> String {
        let uniffi_trait_methods = e.uniffi_trait_methods();

        let mut conformances = vec!["Sendable"];

        if !contains_object_references || uniffi_trait_methods.eq_eq.is_some() {
            conformances.push("Equatable");
        }

        if !contains_object_references || uniffi_trait_methods.hash_hash.is_some() {
            conformances.push("Hashable");
        }

        // Objects can't be Codable at the moment, so we can't derive `Codable` conformance if this Error references one
        if !contains_object_references && self.generate_codable_conformance() {
            conformances.push("Codable");
        }

        if !self.omit_localized_error_conformance() {
            conformances.push("Foundation.LocalizedError");
        }

        if self.generate_case_iterable_conformance() && e.is_flat() && !e.contains_variant_fields()
        {
            conformances.push("CaseIterable");
        }

        conformances.join(", ")
    }

    pub fn conformance_list_for_object(&self, _o: &Object, is_error: &bool) -> String {
        let mut conformances = vec!["@unchecked Sendable"];

        if *is_error {
            conformances.push("Swift.Error");

            if !self.omit_localized_error_conformance() {
                conformances.push("Foundation.LocalizedError");
            }
        }

        conformances.join(", ")
    }
}

// Given a trait, work out what the protocol name we generate for it.
// This differs based on whether the trait supports foreign impls (ie,
// whether is has a "callback interface".
fn trait_protocol_name(ci: &ComponentInterface, trait_ty: &Type) -> Result<String> {
    let Some(module_path) = &trait_ty.module_path() else {
        anyhow::bail!("Invalid trait_type: {trait_ty:?}");
    };
    let Some(ci_look) = ci.find_component_interface(module_path) else {
        anyhow::bail!("no interface with module_path: {}", module_path);
    };

    let (obj_name, has_callback_interface) = match trait_ty {
        Type::Object { name, .. } => {
            let Some(obj) = ci_look.get_object_definition(name) else {
                anyhow::bail!("trait interface not found: {}", name);
            };
            (name, obj.has_callback_interface())
        }
        Type::CallbackInterface { name, .. } => (name, true),
        _ => {
            anyhow::bail!("Invalid trait_type: {trait_ty:?}")
        }
    };

    let class_name = SwiftCodeOracle.class_name(obj_name);
    if has_callback_interface {
        Ok(class_name)
    } else {
        Ok(format!("{class_name}Protocol"))
    }
}

/// Generate UniFFI component bindings for Swift, as strings in memory.
pub fn generate_bindings(config: &Config, ci: &ComponentInterface) -> Result<Bindings> {
    let header = BridgingHeader::new(config, ci)
        .render()
        .context("failed to render Swift bridging header")?;
    let library = SwiftWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render Swift library")?;
    let modulemap = if config.generate_module_map() {
        Some(
            ModuleMap::new_for_single_component(config, ci)
                .render()
                .context("failed to render Swift modulemap")?,
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

/// Generate the bridging header for a component
pub fn generate_header(config: &Config, ci: &ComponentInterface) -> Result<String> {
    BridgingHeader::new(config, ci)
        .render()
        .context("failed to render Swift bridging header")
}

/// Generate the swift source for a component
pub fn generate_swift(config: &Config, ci: &ComponentInterface) -> Result<String> {
    SwiftWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render Swift library")
}

/// Generate the modulemap for a set of components
pub fn generate_modulemap(
    module_name: String,
    header_filenames: Vec<String>,
    xcframework: bool,
    link_frameworks: Vec<String>,
) -> Result<String> {
    ModuleMap {
        module_name,
        header_filenames,
        xcframework,
        link_frameworks,
    }
    .render()
    .context("failed to render Swift library")
}

/// Renders Swift helper code for all types
///
/// This template is a bit different than others in that it stores internal state from the render
/// process.  Make sure to only call `render()` once.
#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "Types.swift")]
pub struct TypeRenderer<'a> {
    config: &'a Config,
    ci: &'a ComponentInterface,
    // Track imports added with the `add_import()` macro
    imports: RefCell<BTreeSet<String>>,
}

impl<'a> TypeRenderer<'a> {
    fn new(config: &'a Config, ci: &'a ComponentInterface) -> Self {
        Self {
            config,
            ci,
            imports: RefCell::new(BTreeSet::new()),
        }
    }

    // The following methods are used by the `Types.swift` macros.

    // Helper to add an import statement
    //
    // Call this inside your template to cause an import statement to be added at the top of the
    // file.  Imports will be sorted and de-deuped.
    //
    // Returns an empty string so that it can be used inside an askama `{{ }}` block.
    fn add_import(&self, name: &str) -> &str {
        self.imports.borrow_mut().insert(name.to_owned());
        ""
    }
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
pub struct ModuleMap {
    module_name: String,
    header_filenames: Vec<String>,
    xcframework: bool,
    link_frameworks: Vec<String>,
}

impl ModuleMap {
    pub fn new_for_single_component(config: &Config, _ci: &ComponentInterface) -> Self {
        Self {
            module_name: config.ffi_module_name(),
            header_filenames: vec![config.header_filename()],
            xcframework: false,
            link_frameworks: config.link_frameworks(),
        }
    }
}

#[derive(Template)]
#[template(syntax = "swift", escape = "none", path = "wrapper.swift")]
pub struct SwiftWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    type_helper_code: String,
    type_imports: BTreeSet<String>,
    ensure_init_fn_name: String,
}
impl<'a> SwiftWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        let type_renderer = TypeRenderer::new(&config, ci);
        let type_helper_code = type_renderer.render().unwrap();
        let type_imports = type_renderer.imports.into_inner();
        Self {
            config,
            ci,
            type_helper_code,
            type_imports,
            ensure_init_fn_name: format!(
                "uniffiEnsure{}Initialized",
                ci.crate_name().to_upper_camel_case()
            ),
        }
    }

    pub fn imports(&self) -> Vec<String> {
        self.type_imports.iter().cloned().collect()
    }

    pub fn initialization_fns(&self) -> Vec<String> {
        let init_fns = self
            .ci
            .iter_local_types()
            .map(|t| SwiftCodeOracle.find(t))
            .filter_map(|ct| ct.initialization_fn());

        // Also call global initialization function for any external type we use.
        // For example, we need to make sure that all callback interface vtables are registered
        // (#2343).
        let extern_module_init_fns = self
            .ci
            .iter_external_types()
            .filter_map(|t| t.module_path())
            .map(|module_path| {
                format!(
                    "uniffiEnsure{}Initialized",
                    module_path.to_upper_camel_case()
                )
            })
            // Collect into a btree set to de-dup and order
            .collect::<BTreeSet<_>>();

        init_fns.chain(extern_module_init_fns).collect()
    }
}

#[derive(Clone)]
pub struct SwiftCodeOracle;

impl SwiftCodeOracle {
    // Map `Type` instances to a `Box<dyn CodeType>` for that type.
    //
    // There is a companion match in `templates/Types.swift` which performs a similar function for the
    // template code.
    //
    //   - When adding additional types here, make sure to also add a match arm to the `Types.swift` template.
    //   - To keep things manageable, let's try to limit ourselves to these 2 mega-matches
    #[allow(clippy::only_used_in_recursion)]
    fn create_code_type(&self, type_: Type) -> Box<dyn CodeType> {
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
            Type::Bytes => Box::new(primitives::BytesCodeType),

            Type::Timestamp => Box::new(miscellany::TimestampCodeType),
            Type::Duration => Box::new(miscellany::DurationCodeType),

            Type::Enum { name, .. } => Box::new(enum_::EnumCodeType::new(name)),
            Type::Object { name, imp, .. } => Box::new(object::ObjectCodeType::new(name, imp)),
            Type::Record { name, .. } => Box::new(record::RecordCodeType::new(name)),
            Type::CallbackInterface { name, .. } => {
                Box::new(callback_interface::CallbackInterfaceCodeType::new(name))
            }
            Type::Optional { inner_type } => {
                Box::new(compounds::OptionalCodeType::new(*inner_type))
            }
            Type::Sequence { inner_type } => {
                Box::new(compounds::SequenceCodeType::new(*inner_type))
            }
            Type::Map {
                key_type,
                value_type,
            } => Box::new(compounds::MapCodeType::new(*key_type, *value_type)),
            Type::Custom { name, builtin, .. } => Box::new(custom::CustomCodeType::new(
                name,
                self.create_code_type(*builtin),
            )),
        }
    }

    fn find(&self, type_: &Type) -> Box<dyn CodeType> {
        self.create_code_type(type_.clone())
    }

    /// Get the idiomatic Swift rendering of a class name (for enums, records, errors, etc).
    fn class_name<S: AsRef<str>>(&self, nm: S) -> String {
        nm.as_ref().to_string().to_upper_camel_case()
    }

    /// Get the idiomatic Swift rendering of a function name.
    fn fn_name<S: AsRef<str>>(&self, nm: S) -> String {
        nm.as_ref().to_string().to_lower_camel_case()
    }

    /// Get the idiomatic Swift rendering of a variable name.
    fn var_name<S: AsRef<str>>(&self, nm: S) -> String {
        nm.as_ref().to_string().to_lower_camel_case()
    }

    /// Get the idiomatic Swift rendering of an individual enum variant.
    fn enum_variant_name<S: AsRef<str>>(&self, nm: S) -> String {
        nm.as_ref().to_string().to_lower_camel_case()
    }

    /// Get the idiomatic Swift rendering of an FFI callback function name
    fn ffi_callback_name<S: AsRef<str>>(&self, nm: S) -> String {
        format!("Uniffi{}", nm.as_ref().to_upper_camel_case())
    }

    /// Get the idiomatic Swift rendering of an FFI struct name
    fn ffi_struct_name<S: AsRef<str>>(&self, nm: S) -> String {
        format!("Uniffi{}", nm.as_ref().to_upper_camel_case())
    }

    /// Get the idiomatic Swift rendering of an if guard name
    fn if_guard_name<S: AsRef<str>>(&self, nm: S) -> String {
        format!("UNIFFI_FFIDEF_{}", nm.as_ref().to_shouty_snake_case())
    }

    fn ffi_type_label(&self, ffi_type: &FfiType) -> String {
        match ffi_type {
            FfiType::Int8 => "Int8".into(),
            FfiType::UInt8 => "UInt8".into(),
            FfiType::Int16 => "Int16".into(),
            FfiType::UInt16 => "UInt16".into(),
            FfiType::Int32 => "Int32".into(),
            FfiType::UInt32 => "UInt32".into(),
            FfiType::Int64 => "Int64".into(),
            FfiType::UInt64 => "UInt64".into(),
            FfiType::Float32 => "Float".into(),
            FfiType::Float64 => "Double".into(),
            FfiType::Handle => "UInt64".into(),
            FfiType::RustBuffer(_) => "RustBuffer".into(),
            FfiType::RustCallStatus => "RustCallStatus".into(),
            FfiType::ForeignBytes => "ForeignBytes".into(),
            // Note: @escaping is required for Swift versions before 5.7 for callbacks passed into
            // async functions. Swift 5.7 and later does not require it.  We should probably remove
            // it once we upgrade our minimum requirement to 5.7 or later.
            FfiType::Callback(name) => format!("@escaping {}", self.ffi_callback_name(name)),
            FfiType::Struct(name) => self.ffi_struct_name(name),
            FfiType::Reference(inner) => {
                format!("UnsafePointer<{}>", self.ffi_type_label(inner))
            }
            FfiType::MutReference(inner) => {
                format!("UnsafeMutablePointer<{}>", self.ffi_type_label(inner))
            }
            FfiType::VoidPointer => "UnsafeMutableRawPointer".into(),
        }
    }

    /// Default values for FFI types
    ///
    /// Used to set a default return value when returning an error
    fn ffi_default_value(&self, return_type: Option<&FfiType>) -> String {
        match return_type {
            Some(t) => match t {
                FfiType::UInt8
                | FfiType::Int8
                | FfiType::UInt16
                | FfiType::Int16
                | FfiType::UInt32
                | FfiType::Int32
                | FfiType::UInt64
                | FfiType::Int64 => "0".to_owned(),
                FfiType::Float32 | FfiType::Float64 => "0.0".to_owned(),
                FfiType::Handle => "0".to_owned(),
                FfiType::RustBuffer(_) => "RustBuffer.empty()".to_owned(),
                _ => unimplemented!("FFI return type: {t:?}"),
            },
            // When we need to use a value for void returns, we use a `u8` placeholder
            None => "0".to_owned(),
        }
    }

    /// Get the name of the protocol and class name for an object.
    ///
    /// If we support callback interfaces, the protocol name is the object name, and the class name is derived from that.
    /// Otherwise, the class name is the object name and the protocol name is derived from that.
    ///
    /// This split determines what types `FfiConverter.lower()` inputs.  If we support callback
    /// interfaces, `lower` must lower anything that implements the protocol.  If not, then lower
    /// only lowers the concrete class.
    fn object_names(&self, obj: &Object) -> (String, String) {
        let class_name = self.class_name(obj.name());
        if obj.has_callback_interface() {
            let impl_name = format!("{class_name}Impl");
            (class_name, impl_name)
        } else {
            (format!("{class_name}Protocol"), class_name)
        }
    }
}

pub mod filters {
    use super::*;
    use uniffi_meta::LiteralMetadata;

    fn oracle() -> &'static SwiftCodeOracle {
        &SwiftCodeOracle
    }

    pub fn type_name(as_type: &impl AsType) -> Result<String, askama::Error> {
        Ok(oracle().find(&as_type.as_type()).type_label())
    }

    pub fn return_type_name(as_type: Option<&impl AsType>) -> Result<String, askama::Error> {
        Ok(match as_type {
            Some(as_type) => oracle().find(&as_type.as_type()).type_label(),
            None => "()".to_owned(),
        })
    }

    pub fn canonical_name(as_type: &impl AsType) -> Result<String, askama::Error> {
        Ok(oracle().find(&as_type.as_type()).canonical_name())
    }

    pub fn ffi_converter_name(as_type: &impl AsType) -> Result<String, askama::Error> {
        Ok(oracle().find(&as_type.as_type()).ffi_converter_name())
    }

    pub fn ffi_error_converter_name(as_type: &impl AsType) -> Result<String, askama::Error> {
        // special handling for types used as errors.
        let mut name = oracle().find(&as_type.as_type()).ffi_converter_name();
        if matches!(&as_type.as_type(), Type::Object { .. }) {
            name.push_str("__as_error")
        }
        Ok(name)
    }

    pub(super) fn ffi_type(type_: &impl AsType) -> askama::Result<FfiType, askama::Error> {
        Ok(type_.as_type().into())
    }

    // To better support external types, we always call the "public" lift and lower functions for
    // "named" types, regardless of whether they are being called from a type in the same crate
    // (ie, a "local" type) or from a different crate (ie, an "external" type)
    pub fn lower_fn(as_type: &impl AsType) -> Result<String, askama::Error> {
        let ty = &as_type.as_type();
        let ffi_converter_name = oracle().find(ty).ffi_converter_name();
        Ok(match ty.name() {
            Some(_) => format!("{}_lower", ffi_converter_name),
            None => format!("{}.lower", ffi_converter_name),
        })
    }

    pub fn write_fn(as_type: &impl AsType) -> Result<String, askama::Error> {
        let ty = &as_type.as_type();
        let ffi_converter_name = oracle().find(ty).ffi_converter_name();
        Ok(format!("{}.write", ffi_converter_name))
    }

    // See above re lower_fn - we always use the public version for named types.
    pub fn lift_fn(as_type: &impl AsType) -> Result<String, askama::Error> {
        let ty = &as_type.as_type();
        let ffi_converter_name = oracle().find(ty).ffi_converter_name();
        Ok(match ty.name() {
            Some(_) => format!("{}_lift", ffi_converter_name),
            None => format!("{}.lift", ffi_converter_name),
        })
    }

    pub fn read_fn(as_type: &impl AsType) -> Result<String, askama::Error> {
        let ty = &as_type.as_type();
        let ffi_converter_name = oracle().find(ty).ffi_converter_name();
        Ok(format!("{}.read", ffi_converter_name))
    }

    pub fn default_swift(
        default: &DefaultValue,
        as_type: &impl AsType,
    ) -> Result<String, askama::Error> {
        Ok(oracle()
            .find(&as_type.as_type())
            .default(default)
            .expect("invalid default: {default:?}"))
    }

    // Get the idiomatic Swift rendering of an individual enum variant's discriminant
    pub fn variant_discr_literal(e: &Enum, index: &usize) -> Result<String, askama::Error> {
        let literal = e.variant_discr(*index).expect("invalid index");
        match literal {
            LiteralMetadata::UInt(v, _, _) => Ok(v.to_string()),
            LiteralMetadata::Int(v, _, _) => Ok(v.to_string()),
            _ => unreachable!("expected an UInt!"),
        }
    }

    /// Get the Swift type for an FFIType
    pub fn ffi_type_name(ffi_type: &FfiType) -> Result<String, askama::Error> {
        Ok(oracle().ffi_type_label(ffi_type))
    }

    pub fn ffi_default_value(return_type: Option<FfiType>) -> Result<String, askama::Error> {
        Ok(oracle().ffi_default_value(return_type.as_ref()))
    }

    /// Like `ffi_type_name`, but used in `BridgingHeaderTemplate.h` which uses a slightly different
    /// names.
    pub fn header_ffi_type_name(ffi_type: &FfiType) -> Result<String, askama::Error> {
        Ok(match ffi_type {
            FfiType::Int8 => "int8_t".into(),
            FfiType::UInt8 => "uint8_t".into(),
            FfiType::Int16 => "int16_t".into(),
            FfiType::UInt16 => "uint16_t".into(),
            FfiType::Int32 => "int32_t".into(),
            FfiType::UInt32 => "uint32_t".into(),
            FfiType::Int64 => "int64_t".into(),
            FfiType::UInt64 => "uint64_t".into(),
            FfiType::Float32 => "float".into(),
            FfiType::Float64 => "double".into(),
            FfiType::Handle => "uint64_t".into(),
            FfiType::RustBuffer(_) => "RustBuffer".into(),
            FfiType::RustCallStatus => "RustCallStatus".into(),
            FfiType::ForeignBytes => "ForeignBytes".into(),
            FfiType::Callback(name) => {
                format!("{} _Nonnull", SwiftCodeOracle.ffi_callback_name(name))
            }
            FfiType::Struct(name) => SwiftCodeOracle.ffi_struct_name(name),
            FfiType::Reference(inner) => {
                format!("const {}* _Nonnull", header_ffi_type_name(inner)?)
            }
            FfiType::MutReference(inner) => format!("{}* _Nonnull", header_ffi_type_name(inner)?),
            FfiType::VoidPointer => "void* _Nonnull".into(),
        })
    }

    /// Get the idiomatic Swift rendering of a class name (for enums, records, errors, etc).
    pub fn class_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Swift rendering of a function name.
    pub fn fn_name(nm: &str) -> Result<String, askama::Error> {
        Ok(quote_general_keyword(oracle().fn_name(nm)))
    }

    /// Get the idiomatic Swift rendering of a variable name.
    pub fn var_name(nm: &str) -> Result<String, askama::Error> {
        Ok(quote_general_keyword(oracle().var_name(nm)))
    }

    /// Get the idiomatic Swift rendering of an arguments name.
    /// This is the same as the var name but quoting is not required.
    pub fn arg_name(nm: &str) -> Result<String, askama::Error> {
        Ok(quote_arg_keyword(oracle().var_name(nm)))
    }

    /// Get the idiomatic Swift rendering of an individual enum variant, quoted if it is a keyword (for use in e.g. declarations)
    pub fn enum_variant_swift_quoted(nm: &str) -> Result<String, askama::Error> {
        Ok(quote_general_keyword(oracle().enum_variant_name(nm)))
    }

    /// Like enum_variant_swift_quoted, but a class name.
    pub fn error_variant_swift_quoted(nm: &str) -> Result<String, askama::Error> {
        Ok(quote_general_keyword(oracle().class_name(nm)))
    }

    /// Get the idiomatic Swift rendering of an FFI callback function name
    pub fn ffi_callback_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().ffi_callback_name(nm))
    }

    /// Get the idiomatic Swift rendering of an FFI struct name
    pub fn ffi_struct_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().ffi_struct_name(nm))
    }

    /// Get the idiomatic Swift rendering of an if guard name
    pub fn if_guard_name(nm: &str) -> Result<String, askama::Error> {
        Ok(oracle().if_guard_name(nm))
    }

    /// Get the idiomatic Swift rendering of docstring
    pub fn docstring(docstring: &str, spaces: &i32) -> Result<String, askama::Error> {
        let middle = textwrap::indent(&textwrap::dedent(docstring), " * ");
        let wrapped = format!("/**\n{middle}\n */");

        let spaces = usize::try_from(*spaces).unwrap_or_default();
        Ok(textwrap::indent(&wrapped, &" ".repeat(spaces)))
    }

    pub fn object_names(obj: &Object) -> Result<(String, String), askama::Error> {
        Ok(SwiftCodeOracle.object_names(obj))
    }
}
