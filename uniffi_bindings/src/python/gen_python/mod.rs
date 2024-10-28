/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{Context, Result};
use rinja::Template;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Debug;

use crate::backend::TemplateExpression;

use uniffi_bindgen::interface::*;
use uniffi_bindgen::VisitMut;

mod callback_interface;
mod compounds;
mod custom;
mod enum_;
mod external;
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

    fn literal(&self, _literal: &Literal) -> String {
        unimplemented!("Unimplemented for {}", self.type_label())
    }

    /// Name of the FfiConverter
    ///
    /// This is the object that contains the lower, write, lift, and read methods for this type.
    fn ffi_converter_name(&self) -> String {
        format!("FfiConverter{}", self.canonical_name())
    }

    /// A list of imports that are needed if this type is in use.
    /// Classes are imported exactly once.
    fn imports(&self) -> Option<Vec<String>> {
        None
    }

    /// Function to run at startup
    fn initialization_fn(&self) -> Option<String> {
        None
    }
}

// Taken from Python's `keyword.py` module.
static KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    let kwlist = vec![
        "False",
        "None",
        "True",
        "__peg_parser__",
        "and",
        "as",
        "assert",
        "async",
        "await",
        "break",
        "class",
        "continue",
        "def",
        "del",
        "elif",
        "else",
        "except",
        "finally",
        "for",
        "from",
        "global",
        "if",
        "import",
        "in",
        "is",
        "lambda",
        "nonlocal",
        "not",
        "or",
        "pass",
        "raise",
        "return",
        "try",
        "while",
        "with",
        "yield",
    ];
    HashSet::from_iter(kwlist.into_iter().map(|s| s.to_string()))
});

// Config options to customize the generated python.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub(super) cdylib_name: Option<String>,
    #[serde(default)]
    custom_types: HashMap<String, CustomTypeConfig>,
    #[serde(default)]
    external_packages: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomTypeConfig {
    // This `CustomTypeConfig` doesn't have a `type_name` like the others -- which is why we have
    // separate structs rather than a shared one.
    imports: Option<Vec<String>>,
    into_custom: TemplateExpression,
    from_custom: TemplateExpression,
}

impl Config {
    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }

    /// Get the package name for a given external namespace.
    pub fn module_for_namespace(&self, ns: &str) -> String {
        let ns = ns.to_string().to_snake_case();
        match self.external_packages.get(&ns) {
            None => format!(".{ns}"),
            Some(value) if value.is_empty() => ns,
            Some(value) => format!("{value}.{ns}"),
        }
    }
}

// Generate python bindings for the given ComponentInterface, as a string.
pub fn generate_python_bindings(config: &Config, ci: &mut ComponentInterface) -> Result<String> {
    PythonWrapper::new(config.clone(), ci)
        .render()
        .context("failed to render python bindings")
}

/// A struct to record a Python import statement.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ImportRequirement {
    /// A simple module import.
    Module { mod_name: String },
    /// A single symbol from a module.
    Symbol {
        mod_name: String,
        symbol_name: String,
    },
    /// A single symbol from a module with the specified local name.
    SymbolAs {
        mod_name: String,
        symbol_name: String,
        as_name: String,
    },
}

impl ImportRequirement {
    /// Render the Python import statement.
    fn render(&self) -> String {
        match &self {
            ImportRequirement::Module { mod_name } => format!("import {mod_name}"),
            ImportRequirement::Symbol {
                mod_name,
                symbol_name,
            } => format!("from {mod_name} import {symbol_name}"),
            ImportRequirement::SymbolAs {
                mod_name,
                symbol_name,
                as_name,
            } => format!("from {mod_name} import {symbol_name} as {as_name}"),
        }
    }
}

/// Renders Python helper code for all types
///
/// This template is a bit different than others in that it stores internal state from the render
/// process.  Make sure to only call `render()` once.
#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "Types.py")]
pub struct TypeRenderer<'a> {
    python_config: &'a Config,
    ci: &'a ComponentInterface,
    // Track included modules for the `include_once()` macro
    include_once_names: RefCell<HashSet<String>>,
    // Track imports added with the `add_import()` macro
    imports: RefCell<BTreeSet<ImportRequirement>>,
}

impl<'a> TypeRenderer<'a> {
    fn new(python_config: &'a Config, ci: &'a ComponentInterface) -> Self {
        Self {
            python_config,
            ci,
            include_once_names: RefCell::new(HashSet::new()),
            imports: RefCell::new(BTreeSet::new()),
        }
    }

    // The following methods are used by the `Types.py` macros.

    // Helper for the including a template, but only once.
    //
    // The first time this is called with a name it will return true, indicating that we should
    // include the template.  Subsequent calls will return false.
    fn include_once_check(&self, name: &str) -> bool {
        self.include_once_names
            .borrow_mut()
            .insert(name.to_string())
    }

    // Helper to add an import statement
    //
    // Call this inside your template to cause an import statement to be added at the top of the
    // file.  Imports will be sorted and de-deuped.
    //
    // Returns an empty string so that it can be used inside an rinja `{{ }}` block.
    fn add_import(&self, name: &str) -> &str {
        self.imports.borrow_mut().insert(ImportRequirement::Module {
            mod_name: name.to_owned(),
        });
        ""
    }

    // Like add_import, but arranges for `from module import name`.
    fn add_import_of(&self, mod_name: &str, name: &str) -> &str {
        self.imports.borrow_mut().insert(ImportRequirement::Symbol {
            mod_name: mod_name.to_owned(),
            symbol_name: name.to_owned(),
        });
        ""
    }

    // Like add_import, but arranges for `from module import name as other`.
    fn add_import_of_as(&self, mod_name: &str, symbol_name: &str, as_name: &str) -> &str {
        self.imports
            .borrow_mut()
            .insert(ImportRequirement::SymbolAs {
                mod_name: mod_name.to_owned(),
                symbol_name: symbol_name.to_owned(),
                as_name: as_name.to_owned(),
            });
        ""
    }

    // An inefficient algo to return type aliases needed for custom types
    // in an order such that dependencies are in the correct order.
    // Eg, if there's a custom type `Guid` -> `str` and another `GuidWrapper` -> `Guid`,
    // it's important the type alias for `Guid` appears first. Fails to handle
    // another level of indirection (eg, `A { builtin: C}, B { }, C { builtin: B })`)
    // but that's pathological :)
    fn get_custom_type_aliases(&self) -> Vec<(String, &Type)> {
        let mut ordered = vec![];
        for type_ in self.ci.iter_types() {
            if let Type::Custom { name, builtin, .. } = type_ {
                match ordered.iter().position(|x: &(&str, &Type)| {
                    x.1.iter_types()
                        .any(|nested_type| *name == nested_type.as_codetype().type_label())
                }) {
                    // This 'name' appears as a builtin, so we must insert our type first.
                    Some(pos) => ordered.insert(pos, (name, builtin)),
                    // Otherwise at the end.
                    None => ordered.push((name, builtin)),
                }
            }
        }
        ordered
            .into_iter()
            .map(|(n, t)| (PythonCodeOracle.class_name(n), t))
            .collect()
    }

    // Sort object types to avoid forward references; traits before everything else.
    fn iter_sorted_object_types(&self) -> impl Iterator<Item = &Type> {
        let mut obs: Vec<&Type> = self
            .ci
            .iter_types()
            .filter(|t| matches!(t, Type::Object { .. }))
            .collect();
        obs.sort_by_key(|t| !matches!(t, Type::Object { imp, .. } if imp.is_trait_interface()));
        obs.into_iter()
    }
}

#[derive(Template)]
#[template(syntax = "py", escape = "none", path = "wrapper.py")]
pub struct PythonWrapper<'a> {
    ci: &'a ComponentInterface,
    config: Config,
    type_helper_code: String,
    type_imports: BTreeSet<ImportRequirement>,
}
impl<'a> PythonWrapper<'a> {
    pub fn new(config: Config, ci: &'a mut ComponentInterface) -> Self {
        ci.visit_mut(&PythonCodeOracle);

        let type_renderer = TypeRenderer::new(&config, ci);
        let type_helper_code = type_renderer.render().unwrap();
        let type_imports = type_renderer.imports.into_inner();

        Self {
            config,
            ci,
            type_helper_code,
            type_imports,
        }
    }

    pub fn imports(&self) -> Vec<ImportRequirement> {
        self.type_imports.iter().cloned().collect()
    }
}

fn fixup_keyword(name: String) -> String {
    if KEYWORDS.contains(&name) {
        format!("_{name}")
    } else {
        name
    }
}

#[derive(Clone, Default)]
pub struct PythonCodeOracle;

impl PythonCodeOracle {
    fn find(&self, type_: &Type) -> Box<dyn CodeType> {
        type_.clone().as_type().as_codetype()
    }

    /// Get the idiomatic Python rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &str) -> String {
        fixup_keyword(nm.to_string().to_upper_camel_case())
    }

    /// Get the idiomatic Python rendering of a function name.
    fn fn_name(&self, nm: &str) -> String {
        fixup_keyword(nm.to_string().to_snake_case())
    }

    /// Get the idiomatic Python rendering of a variable name.
    fn var_name(&self, nm: &str) -> String {
        fixup_keyword(nm.to_string().to_snake_case())
    }

    /// Get the idiomatic Python rendering of an individual enum variant.
    fn enum_variant_name(&self, nm: &str) -> String {
        fixup_keyword(nm.to_string().to_shouty_snake_case())
    }

    /// Get the idiomatic Python rendering of an FFI callback function name
    fn ffi_callback_name(&self, nm: &str) -> String {
        format!("_UNIFFI_{}", nm.to_shouty_snake_case())
    }

    /// Get the idiomatic Python rendering of an FFI struct name
    fn ffi_struct_name(&self, nm: &str) -> String {
        // The ctypes docs use both SHOUTY_SNAKE_CASE AND UpperCamelCase for structs. Let's use
        // UpperCamelCase and reserve shouting for global variables
        format!("_Uniffi{}", nm.to_upper_camel_case())
    }

    fn ffi_type_label(&self, ffi_type: &FfiType) -> String {
        match ffi_type {
            FfiType::Int8 => "ctypes.c_int8".to_string(),
            FfiType::UInt8 => "ctypes.c_uint8".to_string(),
            FfiType::Int16 => "ctypes.c_int16".to_string(),
            FfiType::UInt16 => "ctypes.c_uint16".to_string(),
            FfiType::Int32 => "ctypes.c_int32".to_string(),
            FfiType::UInt32 => "ctypes.c_uint32".to_string(),
            FfiType::Int64 => "ctypes.c_int64".to_string(),
            FfiType::UInt64 => "ctypes.c_uint64".to_string(),
            FfiType::Float32 => "ctypes.c_float".to_string(),
            FfiType::Float64 => "ctypes.c_double".to_string(),
            FfiType::Handle => "ctypes.c_uint64".to_string(),
            FfiType::RustArcPtr(_) => "ctypes.c_void_p".to_string(),
            FfiType::RustBuffer(maybe_external) => match maybe_external {
                Some(external_meta) => format!("_UniffiRustBuffer{}", external_meta.name),
                None => "_UniffiRustBuffer".to_string(),
            },
            FfiType::RustCallStatus => "_UniffiRustCallStatus".to_string(),
            FfiType::ForeignBytes => "_UniffiForeignBytes".to_string(),
            FfiType::Callback(name) => self.ffi_callback_name(name),
            FfiType::Struct(name) => self.ffi_struct_name(name),
            // Pointer to an `asyncio.EventLoop` instance
            FfiType::Reference(inner) => format!("ctypes.POINTER({})", self.ffi_type_label(inner)),
            FfiType::VoidPointer => "ctypes.c_void_p".to_string(),
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
                FfiType::RustArcPtr(_) => "ctypes.c_void_p()".to_owned(),
                FfiType::RustBuffer(maybe_external) => match maybe_external {
                    Some(external_meta) => {
                        format!("_UniffiRustBuffer{}.default()", external_meta.name)
                    }
                    None => "_UniffiRustBuffer.default()".to_owned(),
                },
                _ => unimplemented!("FFI return type: {t:?}"),
            },
            // When we need to use a value for void returns, we use a `u8` placeholder
            None => "0".to_owned(),
        }
    }
}

impl VisitMut for PythonCodeOracle {
    fn visit_record(&self, record: &mut Record) {
        record.rename(self.class_name(record.name()));
    }

    fn visit_object(&self, object: &mut Object) {
        object.rename(self.class_name(object.name()));
        for i in object.trait_impls_mut() {
            i.trait_name = self.class_name(&i.trait_name);
            // should i.tr_module_path be fixed?
        }
    }

    fn visit_field(&self, field: &mut Field) {
        field.rename(self.var_name(field.name()));
    }

    fn visit_ffi_field(&self, ffi_field: &mut FfiField) {
        ffi_field.rename(self.var_name(ffi_field.name()));
    }

    fn visit_ffi_argument(&self, ffi_argument: &mut FfiArgument) {
        ffi_argument.rename(self.class_name(ffi_argument.name()));
    }

    fn visit_enum(&self, _is_error: bool, enum_: &mut Enum) {
        enum_.rename(self.class_name(enum_.name()));
    }

    fn visit_enum_key(&self, key: &mut String) -> String {
        self.class_name(key)
    }

    fn visit_variant(&self, is_error: bool, variant: &mut Variant) {
        if is_error {
            variant.rename(self.class_name(variant.name()));
        } else {
            variant.rename(self.enum_variant_name(variant.name()));
        }
    }

    fn visit_type(&self, type_: &mut Type) {
        // Renaming Types is a special case. We have simple types with names like
        // an Object, but we also have types which have inner_types and builtin types.
        // Which in turn have a different name. Therefore we pass the patterns as a
        // function down to the renaming operation of the type itself, which can apply it
        // to all its nested names if needed.
        let name_transformer = |name: &str| self.class_name(name);
        type_.rename_recursive(&name_transformer);
    }

    fn visit_method(&self, method: &mut Method) {
        method.rename(self.fn_name(method.name()));
    }

    fn visit_argument(&self, argument: &mut Argument) {
        argument.rename(self.var_name(argument.name()));
    }

    fn visit_constructor(&self, constructor: &mut Constructor) {
        if !constructor.is_primary_constructor() {
            constructor.rename(self.fn_name(constructor.name()));
        }
    }

    fn visit_function(&self, function: &mut Function) {
        // Conversions for wrapper.py
        //TODO: Renaming the function name in wrapper.py is not currently tested
        function.rename(self.fn_name(function.name()));
    }

    fn visit_error_name(&self, name: &mut String) {
        *name = self.class_name(name);
    }
}

trait AsCodeType {
    fn as_codetype(&self) -> Box<dyn CodeType>;
}

impl<T: AsType> AsCodeType for T {
    fn as_codetype(&self) -> Box<dyn CodeType> {
        // Map `Type` instances to a `Box<dyn CodeType>` for that type.
        //
        // There is a companion match in `templates/Types.py` which performs a similar function for the
        // template code.
        //
        //   - When adding additional types here, make sure to also add a match arm to the `Types.py` template.
        //   - To keep things manageable, let's try to limit ourselves to these 2 mega-matches
        match self.as_type() {
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
            Type::Object { name, .. } => Box::new(object::ObjectCodeType::new(name)),
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
            Type::External { name, .. } => Box::new(external::ExternalCodeType::new(name)),
            Type::Custom { name, .. } => Box::new(custom::CustomCodeType::new(name)),
        }
    }
}

pub mod filters {
    use super::*;
    pub use crate::backend::filters::*;

    pub(super) fn type_name(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(as_ct.as_codetype().type_label())
    }

    pub(super) fn ffi_converter_name(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(String::from("_Uniffi") + &as_ct.as_codetype().ffi_converter_name()[3..])
    }

    pub(super) fn canonical_name(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(as_ct.as_codetype().canonical_name())
    }

    pub(super) fn lift_fn(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(format!("{}.lift", ffi_converter_name(as_ct)?))
    }

    pub(super) fn check_lower_fn(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(format!("{}.check_lower", ffi_converter_name(as_ct)?))
    }

    pub(super) fn lower_fn(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(format!("{}.lower", ffi_converter_name(as_ct)?))
    }

    pub(super) fn read_fn(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(format!("{}.read", ffi_converter_name(as_ct)?))
    }

    pub(super) fn write_fn(as_ct: &impl AsCodeType) -> Result<String, rinja::Error> {
        Ok(format!("{}.write", ffi_converter_name(as_ct)?))
    }

    pub(super) fn literal_py(
        literal: &Literal,
        as_ct: &impl AsCodeType,
    ) -> Result<String, rinja::Error> {
        Ok(as_ct.as_codetype().literal(literal))
    }

    // Get the idiomatic Python rendering of an individual enum variant's discriminant
    pub fn variant_discr_literal(e: &Enum, index: &usize) -> Result<String, rinja::Error> {
        let literal = e.variant_discr(*index).expect("invalid index");
        Ok(Type::UInt64.as_codetype().literal(&literal))
    }

    pub fn ffi_type_name(type_: &FfiType) -> Result<String, rinja::Error> {
        Ok(PythonCodeOracle.ffi_type_label(type_))
    }

    pub fn ffi_default_value(return_type: Option<FfiType>) -> Result<String, rinja::Error> {
        Ok(PythonCodeOracle.ffi_default_value(return_type.as_ref()))
    }

    /// Get the idiomatic Python rendering of an FFI callback function name
    pub fn ffi_callback_name(nm: &str) -> Result<String, rinja::Error> {
        Ok(PythonCodeOracle.ffi_callback_name(nm))
    }

    /// Get the idiomatic Python rendering of an FFI struct name
    pub fn ffi_struct_name(nm: &str) -> Result<String, rinja::Error> {
        Ok(PythonCodeOracle.ffi_struct_name(nm))
    }

    /// Get the idiomatic Python rendering of docstring
    pub fn docstring(docstring: &str, spaces: &i32) -> Result<String, rinja::Error> {
        let docstring = textwrap::dedent(docstring);
        // Escape triple quotes to avoid syntax error
        let escaped = docstring.replace(r#"""""#, r#"\"\"\""#);

        let wrapped = format!("\"\"\"\n{escaped}\n\"\"\"");

        let spaces = usize::try_from(*spaces).unwrap_or_default();
        Ok(textwrap::indent(&wrapped, &" ".repeat(spaces)))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_docstring_escape() {
        let docstring = r#""""This is a docstring beginning with triple quotes.
Contains "quotes" in it.
It also has a triple quote: """
And a even longer quote: """"""#;

        let expected = r#""""
\"\"\"This is a docstring beginning with triple quotes.
Contains "quotes" in it.
It also has a triple quote: \"\"\"
And a even longer quote: \"\"\"""
""""#;

        assert_eq!(super::filters::docstring(docstring, &0).unwrap(), expected);
    }
}
