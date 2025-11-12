/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use indexmap::IndexMap;
use uniffi_pipeline::Node;

/// Root node of the Initial IR
#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub struct Root {
    pub namespaces: IndexMap<String, Namespace>,
    /// The library path the user passed to us, if we're in library mode
    pub cdylib: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub struct Namespace {
    pub name: String,
    pub crate_name: String,
    /// contents of the `uniffi.toml` file for this module, if present
    pub config_toml: Option<String>,
    pub docstring: Option<String>,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(FnMetadata))]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub enum TypeDefinition {
    Interface(Interface),
    CallbackInterface(CallbackInterface),
    Record(Record),
    Enum(Enum),
    Custom(CustomType),
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(ConstructorMetadata))]
pub struct Constructor {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(MethodMetadata))]
pub struct Method {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(TraitMethodMetadata))]
pub struct TraitMethod {
    pub trait_name: String,
    // Note: the position of `index` is important since it causes callback interface methods to be
    // ordered correctly in MetadataGroup.items
    pub index: u32,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(FnParamMetadata))]
pub struct Argument {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
    pub default: Option<DefaultValue>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(DefaultValueMetadata))]
pub enum DefaultValue {
    Default,
    Literal(Literal),
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(LiteralMetadata))]
pub enum Literal {
    Boolean(bool),
    String(String),
    // Integers are represented as the widest representation we can.
    // Number formatting vary with language and radix, so we avoid a lot of parsing and
    // formatting duplication by using only signed and unsigned variants.
    UInt(u64, Radix, Type),
    Int(i64, Radix, Type),
    // Pass the string representation through as typed in the UDL.
    // This avoids a lot of uncertainty around precision and accuracy,
    // though bindings for languages less sophisticated number parsing than WebIDL
    // will have to do extra work.
    Float(String, Type),
    Enum(String, Type),
    EmptySequence,
    EmptyMap,
    None,
    Some { inner: Box<DefaultValue> },
}

// Represent the radix of integer literal values.
// We preserve the radix into the generated bindings for readability reasons.
#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub enum Radix {
    Decimal = 10,
    Octal = 8,
    Hexadecimal = 16,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(RecordMetadata))]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(FieldMetadata))]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub default: Option<DefaultValue>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub enum EnumShape {
    Enum,
    Error { flat: bool },
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(EnumMetadata))]
pub struct Enum {
    pub name: String,
    pub shape: EnumShape,
    pub variants: Vec<Variant>,
    pub discr_type: Option<Type>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(VariantMetadata))]
pub struct Variant {
    pub name: String,
    pub discr: Option<Literal>,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(ObjectMetadata))]
pub struct Interface {
    pub name: String,
    pub docstring: Option<String>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub trait_impls: Vec<ObjectTraitImpl>,
    pub imp: ObjectImpl,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(CallbackInterfaceMetadata))]
pub struct CallbackInterface {
    pub name: String,
    pub docstring: Option<String>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(UniffiTraitMetadata))]
pub enum UniffiTrait {
    Debug { fmt: Method },
    Display { fmt: Method },
    Eq { eq: Method, ne: Method },
    Hash { hash: Method },
    Ord { cmp: Method },
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(ObjectTraitImplMetadata))]
pub struct ObjectTraitImpl {
    pub ty: Type,
    pub trait_ty: Type,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
#[node(from(CustomTypeMetadata))]
pub struct CustomType {
    pub name: String,
    pub builtin: Type,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub enum Type {
    // Primitive types.
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Float32,
    Float64,
    Boolean,
    String,
    Bytes,
    Timestamp,
    Duration,
    // Structurally recursive types.
    Optional {
        inner_type: Box<Type>,
    },
    Sequence {
        inner_type: Box<Type>,
    },
    Map {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    // User defined types in the API
    #[node(from(Object))]
    Interface {
        module_path: String, // from the metadata
        namespace: String,   // we'll fix this up.
        name: String,
        imp: ObjectImpl,
    },
    Record {
        module_path: String,
        namespace: String,
        name: String,
    },
    Enum {
        module_path: String,
        namespace: String,
        name: String,
    },
    CallbackInterface {
        module_path: String,
        namespace: String,
        name: String,
    },
    Custom {
        module_path: String,
        namespace: String,
        name: String,
        builtin: Box<Type>,
    },
}

#[derive(Debug, Clone, Node, PartialEq, Eq)]
pub enum ObjectImpl {
    // A single Rust type
    Struct,
    // A trait that's can be implemented by Rust types
    Trait,
    // A trait + a callback interface -- can be implemented by both Rust and foreign types.
    CallbackTrait,
}
