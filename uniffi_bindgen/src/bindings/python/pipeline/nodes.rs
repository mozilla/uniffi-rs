/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use askama::Template;
use heck::ToSnakeCase;

use uniffi_pipeline::{MapNode, Node};

use crate::{bindings::python::filters, pipeline::general};

use super::*;

use_prev_node!(general::AsyncData);
use_prev_node!(general::Checksum);
use_prev_node!(general::EnumShape);
use_prev_node!(general::FieldsKind);
use_prev_node!(general::FfiFunctionKind);
use_prev_node!(
    general::FfiFunctionTypeName,
    names::map_ffi_function_type_name
);
use_prev_node!(general::FfiStructName, names::map_ffi_struct_name);
use_prev_node!(general::FfiType);
use_prev_node!(general::HandleKind);
use_prev_node!(general::ObjectImpl);
use_prev_node!(general::Radix);
use_prev_node!(general::RustFfiFunctionName);
use_prev_node!(general::Type, types::map_type);

/// Initial IR, this stores the metadata and other data
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Root))]
#[map_node(update_context(context.update_from_root(&self)))]
pub struct Root {
    /// In library mode, the library path the user passed to us
    pub cdylib: Option<String>,
    #[map_node(from(namespaces))]
    pub modules: IndexMap<String, Module>,
}

#[derive(Debug, Clone, Node, MapNode, Template)]
#[template(syntax = "py", escape = "none", path = "Module.py")]
#[map_node(from(general::Namespace))]
#[map_node(modules::map_namespace)]
pub struct Module {
    pub cdylib_name: String,
    pub has_async_fns: bool,
    pub has_callback_interface: bool,
    pub has_async_callback_method: bool,
    pub imports: Vec<String>,
    pub exported_names: Vec<String>,
    pub name: String,
    pub crate_name: String,
    pub docstring: Option<String>,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
    pub ffi_definitions: IndexSet<FfiDefinition>,
    pub checksums: Vec<Checksum>,
    pub ffi_rustbuffer_alloc: RustFfiFunctionName,
    pub ffi_rustbuffer_from_bytes: RustFfiFunctionName,
    pub ffi_rustbuffer_free: RustFfiFunctionName,
    pub ffi_rustbuffer_reserve: RustFfiFunctionName,
    pub ffi_uniffi_contract_version: RustFfiFunctionName,
    // Correct contract version value
    pub correct_contract_version: String,
    pub string_type_node: TypeNode,
}

// These structs exist so that we can easily deserialize the entire `uniffi.toml` file.
// We then extract the `PythonConfig`, which is what we actually care about.

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Function))]
pub struct Function {
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::TypeDefinition))]
pub enum TypeDefinition {
    Interface(Interface),
    CallbackInterface(CallbackInterface),
    Record(Record),
    Enum(Enum),
    Custom(CustomType),
    /// Type that doesn't contain any other type
    Simple(TypeNode),
    /// Compound types
    Optional(OptionalType),
    Sequence(SequenceType),
    Map(MapType),
    /// User types that are defined in another crate
    External(ExternalType),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Constructor))]
pub struct Constructor {
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::Method))]
pub struct Method {
    pub callable: Callable,
    pub docstring: Option<String>,
}

/// Common data from Function/Method/Constructor
#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::Callable))]
pub struct Callable {
    #[map_node(callables::name(&self))]
    pub name: String,
    pub async_data: Option<AsyncData>,
    pub kind: CallableKind,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
    pub throws_type: ThrowsType,
    pub checksum: Option<u16>,
    pub ffi_func: RustFfiFunctionName,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::CallableKind))]
pub enum CallableKind {
    /// Toplevel function
    Function,
    /// Interface/Trait interface method
    Method { self_type: TypeNode },
    /// Interface constructor
    Constructor { self_type: TypeNode, primary: bool },
    /// Method inside a VTable or a CallbackInterface
    ///
    /// For trait interfaces this only applies to the Callables inside the `vtable.methods` field.
    /// Callables inside `Interface::methods` will still be `Callable::Method`.
    VTableMethod { self_type: TypeNode },
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::ReturnType))]
#[map_node(types::map_return_type)]
pub struct ReturnType {
    pub ty: Option<TypeNode>,
    pub type_name: String,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::ThrowsType))]
pub struct ThrowsType {
    #[map_node(error::is_from_interface(&self))]
    pub from_interface: bool,
    pub ty: Option<TypeNode>,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::Argument))]
pub struct Argument {
    #[map_node(names::var_name(&self.name))]
    pub name: String,
    pub ty: TypeNode,
    pub optional: bool,
    pub default: Option<DefaultValueNode>,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::DefaultValue))]
pub enum DefaultValue {
    Default(TypeNode),
    Literal(LiteralNode),
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::DefaultValue))]
pub struct DefaultValueNode {
    /// The default value rendered as a Python string
    #[map_node(default::render_default(&self, context)?)]
    pub py_default: String,
    /// The default value as specified as a literal in function args.
    #[map_node(default::arg_literal(&self, context)?)]
    pub arg_literal: String,
    #[map_node(self.map_node(context)?)]
    pub default: DefaultValue,
}

impl DefaultValueNode {
    fn is_arg_literal(&self) -> bool {
        self.py_default == self.arg_literal
    }
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::Literal))]
pub struct LiteralNode {
    /// The literal rendered as a Python string
    #[map_node(default::render_literal(&self, context)?)]
    pub py_lit: String,
    #[map_node(self.map_node(context)?)]
    pub lit: Literal,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::Literal))]
pub enum Literal {
    Boolean(bool),
    String(String),
    // Integers are represented as the widest representation we can.
    // Number formatting vary with language and radix, so we avoid a lot of parsing and
    // formatting duplication by using only signed and unsigned variants.
    UInt(u64, Radix, TypeNode),
    Int(i64, Radix, TypeNode),
    // Pass the string representation through as typed in the UDL.
    // This avoids a lot of uncertainty around precision and accuracy,
    // though bindings for languages less sophisticated number parsing than WebIDL
    // will have to do extra work.
    Float(String, TypeNode),
    Enum(
        #[map_node(enums::enum_variant_name(&var0, &var1)?)] String,
        TypeNode,
    ),
    EmptySequence,
    EmptyMap,
    None,
    Some {
        inner: Box<DefaultValue>,
    },
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Record))]
pub struct Record {
    #[map_node(names::type_name(&self.name))]
    pub name: String,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
    #[map_node(interfaces::map_constructors(&self.name, self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_trait_methods: UniffiTraitMethods,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Field))]
pub struct Field {
    #[map_node(names::var_name(&self.name))]
    pub name: String,
    pub ty: TypeNode,
    pub default: Option<DefaultValueNode>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Enum))]
pub struct Enum {
    #[map_node(names::type_name(&self.name))]
    pub name: String,
    /// Is this a "flat" enum -- one with no associated data
    pub is_flat: bool,
    pub shape: EnumShape,
    #[map_node(enums::map_variants(self.variants, self.shape, context)?)]
    pub variants: Vec<Variant>,
    pub discr_type: TypeNode,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
    #[map_node(interfaces::map_constructors(&self.name, self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_trait_methods: UniffiTraitMethods,
}

#[derive(Debug, Clone, Node)]
pub struct Variant {
    pub name: String,
    pub discr: LiteralNode,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Interface))]
pub struct Interface {
    #[map_node(interfaces::name(&self))]
    pub name: String,
    #[map_node(interfaces::base_classes(&self, context)?)]
    pub base_classes: Vec<String>,
    #[map_node(interfaces::protocol(&self, context)?)]
    pub protocol: Protocol,
    pub docstring: Option<String>,
    #[map_node(interfaces::map_constructors(&self.name, self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_trait_methods: UniffiTraitMethods,
    pub trait_impls: Vec<ObjectTraitImpl>,
    pub imp: ObjectImpl,
    pub self_type: TypeNode,
    pub vtable: Option<VTable>,
    pub ffi_func_clone: RustFfiFunctionName,
    pub ffi_func_free: RustFfiFunctionName,
}

impl Interface {
    fn has_primary_constructor(&self) -> bool {
        self.has_descendant(|c: &Callable| c.is_primary_constructor())
    }
}

#[derive(Debug, Clone, Node)]
pub struct Protocol {
    pub name: String,
    pub base_classes: Vec<String>,
    pub docstring: Option<String>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::CallbackInterface))]
pub struct CallbackInterface {
    #[map_node(callback_interfaces::callback_interface_name(&self))]
    pub name: String,
    #[map_node(callback_interfaces::protocol(&self, context)?)]
    pub protocol: Protocol,
    pub docstring: Option<String>,
    pub vtable: VTable,
    pub methods: Vec<Method>,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::VTable))]
pub struct VTable {
    pub struct_type: FfiTypeNode,
    pub interface_name: String,
    pub init_fn: RustFfiFunctionName,
    pub clone_fn_type: FfiFunctionTypeName,
    pub free_fn_type: FfiFunctionTypeName,
    pub methods: Vec<VTableMethod>,
}

/// Single method in a vtable
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::VTableMethod))]
pub struct VTableMethod {
    #[map_node(ffi_types::ffi_default_value(&self.callable.return_type, context)?)]
    pub ffi_default_value: String,
    pub callable: Callable,
    pub ffi_type: FfiTypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::ObjectTraitImpl))]
pub struct ObjectTraitImpl {
    pub ty: TypeNode,
    pub trait_ty: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::CustomType))]
pub struct CustomType {
    #[map_node(names::type_name(&self.name))]
    pub name: String,
    #[map_node(context.custom_type_config(&self)?)]
    pub config: Option<CustomTypeConfig>,
    pub builtin: TypeNode,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
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

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::ExternalType))]
pub struct ExternalType {
    pub namespace: String,
    pub name: String,
    pub self_type: TypeNode,
}

/// Wrap `Type` so that we can add extra fields that are set for all variants.
#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::TypeNode))]
pub struct TypeNode {
    #[map_node(types::type_name(&self.ty, context)?)]
    pub type_name: String,
    #[map_node(types::ffi_converter_name(&self, context)?)]
    pub ffi_converter_name: String,
    pub ty: Type,
    pub canonical_name: String,
    pub is_used_as_error: bool,
    pub ffi_type: FfiTypeNode,
}

/// Like `TypeNode` but for FFI types.
///
/// This exists so that language bindings generators can add extra fields
#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(FfiType))]
pub struct FfiTypeNode {
    #[map_node(ffi_types::ffi_type_name(&self, context)?)]
    pub type_name: String,
    #[map_node(self.map_node(context)?)]
    pub ty: FfiType,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::UniffiTraitMethods))]
pub struct UniffiTraitMethods {
    pub debug_fmt: Option<Method>,
    pub display_fmt: Option<Method>,
    pub eq_eq: Option<Method>,
    pub eq_ne: Option<Method>,
    pub hash_hash: Option<Method>,
    pub ord_cmp: Option<Method>,
}

#[derive(Debug, Clone, Node, MapNode, Eq, PartialEq, Hash)]
#[map_node(from(general::FfiDefinition))]
pub enum FfiDefinition {
    /// FFI Function exported in the Rust library
    RustFunction(FfiFunction),
    /// FFI Function definition used in the interface, language, for example a callback interface method.
    FunctionType(FfiFunctionType),
    /// Struct definition used in the interface, for example a callback interface Vtable.
    Struct(FfiStruct),
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiFunction))]
pub struct FfiFunction {
    pub name: RustFfiFunctionName,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
    pub kind: FfiFunctionKind,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiFunctionType))]
pub struct FfiFunctionType {
    pub name: FfiFunctionTypeName,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiReturnType))]
pub struct FfiReturnType {
    pub ty: Option<FfiTypeNode>,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiStruct))]
pub struct FfiStruct {
    pub name: FfiStructName,
    pub fields: Vec<FfiField>,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiField))]
pub struct FfiField {
    pub name: String,
    pub ty: FfiTypeNode,
}

#[derive(Debug, Clone, Node, MapNode, PartialEq, Eq, Hash)]
#[map_node(from(general::FfiArgument))]
pub struct FfiArgument {
    pub name: String,
    pub ty: FfiTypeNode,
}

impl Callable {
    pub fn is_async(&self) -> bool {
        self.async_data.is_some()
    }

    pub fn is_method(&self) -> bool {
        matches!(self.kind, CallableKind::Method { .. })
    }

    pub fn self_type(&self) -> Option<TypeNode> {
        match &self.kind {
            CallableKind::Method { self_type, .. } => Some(self_type.clone()),
            _ => None,
        }
    }

    pub fn is_primary_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { primary: true, .. })
    }
}

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

impl Variant {
    fn has_unnamed_fields(&self) -> bool {
        matches!(self.fields_kind, FieldsKind::Unnamed)
    }
}
