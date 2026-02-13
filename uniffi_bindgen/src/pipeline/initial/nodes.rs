/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use indexmap::IndexMap;

use super::*;

use_prev_node!(uniffi_meta::EnumShape);
use_prev_node!(uniffi_meta::ObjectImpl);
use_prev_node!(uniffi_meta::Radix);

/// Root node of the Initial IR
#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
pub struct Root {
    pub namespaces: IndexMap<String, Namespace>,
    /// The library path the user passed to us, if we're in library mode
    pub cdylib: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
pub struct Namespace {
    pub name: String,
    pub crate_name: String,
    /// contents of the `uniffi.toml` file for this module, if present
    pub config_toml: Option<String>,
    pub docstring: Option<String>,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::FnMetadata))]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
pub enum TypeDefinition {
    Interface(Interface),
    CallbackInterface(CallbackInterface),
    Record(Record),
    Enum(Enum),
    Custom(CustomType),
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::ConstructorMetadata))]
pub struct Constructor {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::MethodMetadata))]
#[map_node(from(uniffi_meta::TraitMethodMetadata))]
pub struct Method {
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::TraitMethodMetadata))]
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

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::FnParamMetadata))]
pub struct Argument {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
    pub default: Option<DefaultValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::DefaultValueMetadata))]
pub enum DefaultValue {
    Default,
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::LiteralMetadata))]
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

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::RecordMetadata))]
pub struct Record {
    #[map_node(context.constructors_for_type(&self.module_path, &self.name)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(context.methods_for_type(&self.module_path, &self.name)?)]
    pub methods: Vec<Method>,
    #[map_node(context.uniffi_traits_for_type(&self.module_path, &self.name)?)]
    pub uniffi_traits: Vec<UniffiTrait>,
    pub name: String,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::FieldMetadata))]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub default: Option<DefaultValue>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::EnumMetadata))]
pub struct Enum {
    #[map_node(context.constructors_for_type(&self.module_path, &self.name)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(context.methods_for_type(&self.module_path, &self.name)?)]
    pub methods: Vec<Method>,
    #[map_node(context.uniffi_traits_for_type(&self.module_path, &self.name)?)]
    pub uniffi_traits: Vec<UniffiTrait>,
    pub name: String,
    pub shape: EnumShape,
    pub variants: Vec<Variant>,
    pub discr_type: Option<Type>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::VariantMetadata))]
pub struct Variant {
    pub name: String,
    pub discr: Option<Literal>,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::ObjectMetadata))]
pub struct Interface {
    #[map_node(context.constructors_for_type(&self.module_path, &self.name)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(context.methods_for_type(&self.module_path, &self.name)?)]
    pub methods: Vec<Method>,
    #[map_node(context.uniffi_traits_for_type(&self.module_path, &self.name)?)]
    pub uniffi_traits: Vec<UniffiTrait>,
    #[map_node(context.trait_impls_for_type(&self.module_path, &self.name)?)]
    pub trait_impls: Vec<ObjectTraitImpl>,
    pub name: String,
    pub docstring: Option<String>,
    pub imp: ObjectImpl,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::CallbackInterfaceMetadata))]
pub struct CallbackInterface {
    #[map_node(context.methods_for_type(&self.module_path, &self.name)?)]
    pub methods: Vec<Method>,
    pub name: String,
    pub docstring: Option<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::UniffiTraitMetadata))]
pub enum UniffiTrait {
    Debug { fmt: Method },
    Display { fmt: Method },
    Eq { eq: Method, ne: Method },
    Hash { hash: Method },
    Ord { cmp: Method },
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::ObjectTraitImplMetadata))]
pub struct ObjectTraitImpl {
    pub ty: Type,
    pub trait_ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Node, MapNode)]
#[map_node(from(uniffi_meta::CustomTypeMetadata))]
pub struct CustomType {
    pub name: String,
    pub builtin: Type,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
#[map_node(from(uniffi_meta::Type))]
#[map_node(types::map_type)]
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
    Interface {
        namespace: String,
        name: String,
        imp: ObjectImpl,
    },
    Record {
        namespace: String,
        name: String,
    },
    Enum {
        namespace: String,
        name: String,
    },
    CallbackInterface {
        namespace: String,
        name: String,
    },
    Custom {
        namespace: String,
        name: String,
        builtin: Box<Type>,
    },
}

impl TypeDefinition {
    pub fn name(&self) -> &str {
        match &self {
            Self::Record(rec) => &rec.name,
            Self::Enum(en) => &en.name,
            Self::Interface(int) => &int.name,
            Self::CallbackInterface(cbi) => &cbi.name,
            Self::Custom(custom) => &custom.name,
        }
    }
}

impl Type {
    pub fn name(&self) -> Option<&str> {
        match &self {
            Type::Record { name, .. }
            | Type::Enum { name, .. }
            | Type::Interface { name, .. }
            | Type::CallbackInterface { name, .. }
            | Type::Custom { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        match &self {
            Type::Record { namespace, .. }
            | Type::Enum { namespace, .. }
            | Type::Interface { namespace, .. }
            | Type::CallbackInterface { namespace, .. }
            | Type::Custom { namespace, .. } => Some(namespace.as_str()),
            _ => None,
        }
    }
}
