/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use super::*;

uniffi_pipeline::use_prev_node!(general::EnumShape);
uniffi_pipeline::use_prev_node!(general::FieldsKind);
uniffi_pipeline::use_prev_node!(general::ObjectImpl);
uniffi_pipeline::use_prev_node!(general::Radix);
uniffi_pipeline::use_prev_node!(general::Type);

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Root))]
#[map_node(update_context(context.update_from_root(&self)?))]
pub struct Root {
    pub cdylib: Option<String>,
    #[map_node(Vec::from_iter(self.namespaces.map_node(context)?.into_values()))]
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Namespace))]
#[map_node(packages::map_namespace)]
pub struct Package {
    pub name: String,
    pub crate_name: String,
    pub config: Config,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
    pub scaffolding_functions: Vec<ScaffoldingFunction>,
    pub imports: IndexSet<String>,
}

#[derive(Debug, Clone, Node)]
#[allow(clippy::large_enum_variant)]
pub enum TypeDefinition {
    Record(Record),
    Enum(Enum),
    Interface(Interface),
    Class(Class),
    Custom(CustomType),
    Optional(OptionalType),
    Sequence(SequenceType),
    Map(MapType),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Record))]
pub struct Record {
    pub fields_kind: FieldsKind,
    pub self_type: TypeNode,
    #[map_node(context.config()?.record_is_immutable(&self.name))]
    pub immutable: bool,
    pub name: String,
    #[map_node(records::map_fields(self.fields, context)?)]
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Enum))]
#[map_node(update_context(context.update_from_enum(&self)))]
pub struct Enum {
    pub is_flat: bool,
    #[map_node(context.config()?.use_enum_entries())]
    pub use_entries: bool,
    pub self_type: TypeNode,
    pub discr_type: TypeNode,
    pub discr_specified: bool,
    pub variants: Vec<Variant>,
    pub name: String,
    pub shape: EnumShape,
    pub docstring: Option<String>,
    pub recursive: bool,
}

/// Kotlin class
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Interface))]
#[map_node(interfaces::map_class)]
pub struct Class {
    pub name: String,
    pub module_path: String,
    pub self_type: TypeNode,
    pub base_classes: Vec<String>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub docstring: Option<String>,
    pub crate_name: String,
}

/// Kotlin Interface
#[derive(Debug, Clone, Node)]
pub struct Interface {
    pub name: String,
    pub methods: Vec<Method>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::CustomType))]
pub struct CustomType {
    #[map_node(context.config()?.custom_types.get(&self.name).cloned())]
    pub config: Option<CustomTypeConfig>,
    #[map_node(context.current_crate_name()?.to_string())]
    pub crate_name: String,
    pub self_type: TypeNode,
    pub name: String,
    pub builtin: TypeNode,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Variant))]
pub struct Variant {
    #[map_node(enums::variant_name_kt(&self, context)?)]
    pub name_kt: String,
    pub name: String,
    pub discr: LiteralNode,
    pub fields_kind: FieldsKind,
    #[map_node(records::map_fields(self.fields, context)?)]
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct Field {
    pub name: String,
    pub index: usize,
    pub ty: TypeNode,
    pub default: Option<DefaultValueNode>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Constructor))]
pub struct Constructor {
    #[map_node(callables::constructor_jni_method_name(&self, context)?)]
    pub jni_method_name: String,
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Method))]
pub struct Method {
    #[map_node(callables::method_jni_method_name(&self, context)?)]
    pub jni_method_name: String,
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Function))]
pub struct Function {
    #[map_node(callables::function_jni_method_name(&self, context)?)]
    pub jni_method_name: String,
    pub module_path: String,
    pub docstring: Option<String>,
    pub callable: Callable,
}

#[derive(Debug, Clone, Node)]
pub struct ScaffoldingFunction {
    pub jni_method_name: String,
    pub callable: Callable,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Callable))]
#[map_node(callables::map_callable)]
pub struct Callable {
    pub kind: CallableKind,
    pub name: String,
    pub arguments: Vec<Argument>,
    pub return_type: Option<TypeNode>,
    pub fully_qualified_name_rs: String,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::CallableKind))]
pub enum CallableKind {
    Function,
    Method {
        self_type: TypeNode,
    },
    Constructor {
        self_type: TypeNode,
        primary: bool,
    },
    VTableMethod {
        self_type: TypeNode,
        for_callback_interface: bool,
    },
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Argument))]
pub struct Argument {
    pub name: String,
    pub ty: TypeNode,
    pub optional: bool,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::OptionalType))]
pub struct OptionalType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::SequenceType))]
pub struct SequenceType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::MapType))]
pub struct MapType {
    pub key: TypeNode,
    pub value: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
#[map_node(from(general::TypeNode))]
#[map_node(types::map_type_node)]
pub struct TypeNode {
    pub canonical_name: String,
    pub is_used_as_error: bool,
    pub ty: Type,
    pub type_kt: String,
    pub type_rs: String,
    pub read_fn_rs: String,
    pub write_fn_rs: String,
    pub read_fn_kt: String,
    pub write_fn_kt: String,
    // Note: no ffi_type field, we have a very different FFI than the general IR
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::DefaultValue))]
#[map_node(defaults::map_default)]
pub struct DefaultValueNode {
    pub default_kt: String,
    pub default: DefaultValue,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Literal))]
#[map_node(defaults::map_literal)]
pub struct LiteralNode {
    pub lit_kt: String,
    pub lit: Literal,
}

/// Default value for a field/argument
///
/// This sets the arg/field type in the case where the user just specified `default`.
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::DefaultValue))]
pub enum DefaultValue {
    Literal(Literal),
    Default(TypeNode),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Literal))]
pub enum Literal {
    Boolean(bool),
    String(String),
    UInt(u64, Radix, TypeNode),
    Int(i64, Radix, TypeNode),
    Float(String, TypeNode),
    Enum(String, TypeNode),
    EmptySequence,
    EmptyMap,
    None,
    Some { inner: Box<DefaultValue> },
}

impl Root {
    pub fn cdylib_name(&self) -> Result<String> {
        let config_names: IndexSet<_> = self
            .packages
            .iter()
            .filter_map(|p| p.config.cdylib_name.as_deref())
            .collect();
        Ok(match config_names.len() {
            0 => match &self.cdylib {
                Some(name) => name.to_string(),
                None => bail!("Unknown cdylib name.  Use `src:[crate_name]` to generate bindings or set it in a `uniffi.toml` config"),
            }
            1 => config_names.into_iter().next().unwrap().to_string(),
            _ => bail!("Conflicting cdylib names in `uniffi.toml` files: {:?}", Vec::from_iter(config_names)),
        })
    }

    /// Type definitions to generate FFI functions for
    ///
    /// This de-dupes the type definitions for all packages so we only don't generate duplicate
    /// functions for types that may be used in multiple packages like `Vec<u32>`.
    pub fn ffi_type_definitions(&self) -> impl Iterator<Item = &TypeDefinition> {
        let mut seen = HashSet::new();
        self.packages
            .iter()
            .flat_map(|p| &p.type_definitions)
            .filter(move |type_def| {
                seen.insert(match type_def {
                    TypeDefinition::Record(r) => &r.self_type,
                    TypeDefinition::Enum(e) => &e.self_type,
                    TypeDefinition::Optional(o) => &o.self_type,
                    TypeDefinition::Sequence(s) => &s.self_type,
                    TypeDefinition::Map(m) => &m.self_type,
                    TypeDefinition::Class(c) => &c.self_type,
                    TypeDefinition::Custom(c) => &c.self_type,
                    TypeDefinition::Interface(_) => return false,
                })
            })
    }

    pub fn disable_java_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        // https://github.com/mozilla/uniffi-rs/issues/2866 would help here.
        self.packages.iter().any(|p| p.config.disable_java_cleaner)
    }

    pub fn enable_android_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        // https://github.com/mozilla/uniffi-rs/issues/2866 would help here.
        self.packages.iter().any(|p| p.config.android_cleaner())
    }
}

impl Package {
    pub fn jni_class(&self) -> String {
        format!("`{}`", self.crate_name.to_upper_camel_case())
    }

    pub fn name_jni(&self) -> String {
        self.name.replace(".", "/")
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.crate_name)
    }

    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.type_definitions
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Class(cls) => Some(cls),
                _ => None,
            })
    }
}

impl Callable {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn has_receiver(&self) -> bool {
        self.receiver_type().is_some()
    }

    pub fn receiver_type(&self) -> Option<&TypeNode> {
        match &self.kind {
            CallableKind::Method { self_type, .. }
            | CallableKind::VTableMethod { self_type, .. } => Some(self_type),
            _ => None,
        }
    }

    pub fn is_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { .. })
    }

    pub fn is_primary_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { primary: true, .. })
    }

    pub fn arg_list(&self) -> String {
        self.arguments
            .iter()
            .map(|a| format!("{}: {}", a.name_kt(), a.ty.type_kt))
            .collect::<Vec<_>>()
            .join(" , ")
    }

    pub fn return_type_kt(&self) -> &str {
        match &self.return_type {
            None => "Unit",
            Some(ty) => &ty.type_kt,
        }
    }
}

impl Class {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn jni_free_name(&self) -> String {
        format!(
            "objectFree{}{}",
            self.crate_name.to_upper_camel_case(),
            self.name.to_upper_camel_case(),
        )
    }

    pub fn jni_addref_name(&self) -> String {
        format!(
            "objectAddReff{}{}",
            self.crate_name.to_upper_camel_case(),
            self.name.to_upper_camel_case(),
        )
    }

    pub fn primary_constructor(&self) -> Option<&Constructor> {
        self.constructors.iter().find(|c| {
            matches!(
                c.callable.kind,
                CallableKind::Constructor { primary: true, .. }
            )
        })
    }

    pub fn secondary_constructors(&self) -> impl Iterator<Item = &Constructor> {
        self.constructors.iter().filter(|c| {
            matches!(
                c.callable.kind,
                CallableKind::Constructor { primary: false, .. }
            )
        })
    }
}

impl Interface {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }
}

impl Record {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}

impl Enum {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}

impl CustomType {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }
}

impl Variant {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}

impl Field {
    pub fn name_kt(&self) -> String {
        if self.name.is_empty() {
            format!("v{}", self.index + 1)
        } else {
            format!("`{}`", self.name.to_lower_camel_case())
        }
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}
impl Argument {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}
