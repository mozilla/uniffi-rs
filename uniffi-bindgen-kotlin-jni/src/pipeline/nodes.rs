/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

uniffi_pipeline::use_prev_node!(general::EnumShape);
uniffi_pipeline::use_prev_node!(general::FieldsKind);
uniffi_pipeline::use_prev_node!(general::ObjectImpl);
uniffi_pipeline::use_prev_node!(general::Radix);
uniffi_pipeline::use_prev_node!(general::TraitKind);
uniffi_pipeline::use_prev_node!(general::Type);

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Root))]
#[map_node(root::map_root)]
pub struct Root {
    pub cdylib: Option<String>,
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
}

#[derive(Debug, Clone, Node)]
#[allow(clippy::large_enum_variant)]
pub enum TypeDefinition {
    Record(Record),
    Enum(Enum),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Record))]
#[map_node(records::map_record)]
pub struct Record {
    pub self_type: TypeNode,
    pub immutable: bool,
    pub name: String,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Enum))]
#[map_node(enums::map_enum)]
pub struct Enum {
    pub is_flat: bool,
    pub use_entries: bool,
    pub self_type: TypeNode,
    pub discr_type: TypeNode,
    pub discr_specified: bool,
    pub variants: Vec<Variant>,
    pub name: String,
    pub shape: EnumShape,
    pub kotlin_kind: KotlinEnumKind,
    pub docstring: Option<String>,
    pub recursive: bool,
    pub ffi_fields: Vec<FfiField>,
}

#[derive(Debug, Clone, Node)]
pub enum KotlinEnumKind {
    EnumClass { discr_type: Option<String> },
    FlatError,
    SealedClass,
}

#[derive(Debug, Clone, Node)]
pub struct Variant {
    pub name_kt: String,
    pub name: String,
    pub discr: LiteralNode,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub used_ffi_fields: IndexSet<FfiField>,
}

#[derive(Debug, Clone, Node)]
pub struct Field {
    pub name: String,
    pub index: usize,
    pub ty: TypeNode,
    pub default: Option<DefaultValueNode>,
    pub docstring: Option<String>,
    pub ffi_fields: Vec<FfiField>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Function))]
#[map_node(callables::map_function)]
pub struct Function {
    pub jni_method_name: String,
    pub docstring: Option<String>,
    pub callable: Callable,
}

#[derive(Debug, Clone, Node)]
pub struct Callable {
    pub kind: CallableKind,
    pub name: String,
    pub is_async: bool,
    pub fully_qualified_name_rs: String,
    pub arguments: Vec<Argument>,
    pub return_type: Option<TypeNode>,
    pub throws_type: Option<TypeNode>,
    pub return_ffi: ReturnFfi,
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

#[derive(Debug, Clone, Node)]
pub struct Argument {
    pub name: String,
    pub index: usize,
    pub ty: TypeNode,
    pub optional: bool,
    pub ffi_args: Vec<FfiArgument>,
}

/// Wrap `Type` so that we can add extra fields that are set for all variants.
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::TypeNode))]
#[map_node(types::map_type_node)]
pub struct TypeNode {
    pub id: u64,
    pub ty: Type,
    pub type_rs: String,
    pub type_kt: String,
    pub is_used_as_error: bool,
    pub ffi_types: Vec<FfiType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Node)]
pub struct FfiField {
    pub index: usize,
    pub ty: FfiType,
}

/// Argument on the JNI FFI function
#[derive(Debug, Clone, Node, MapNode)]
pub struct FfiArgument {
    pub name: String,
    pub ty: FfiType,
}

#[derive(Debug, Clone, Node)]
pub enum ReturnFfi {
    /// JNI function returns a single primitive value
    Primitive {
        type_node: TypeNode,
        ffi_type: FfiType,
    },
    /// High-level type is deconstructed then returned
    ///
    /// The exact mechanics of this varies by call type, see DESIGN.md for details.
    Deconstruct {
        type_node: TypeNode,
        ffi_types: Vec<FfiType>,
    },
    Void,
}

/// Primitive type that's passed across the FFI using JNI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Node)]
pub enum FfiType {
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
    ByteArray,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Literal))]
#[map_node(defaults::map_literal)]
pub struct LiteralNode {
    pub lit_kt: String,
    pub lit: Literal,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::DefaultValue))]
#[map_node(defaults::map_default)]
pub struct DefaultValueNode {
    pub default_kt: String,
    pub default: DefaultValue,
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
    EmptySet,
    None,
    Some { inner: Box<DefaultValue> },
}
