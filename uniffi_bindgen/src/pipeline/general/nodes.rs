/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use indexmap::{IndexMap, IndexSet};
use uniffi_pipeline::Node;

/// Initial IR, this stores the metadata and other data
#[derive(Debug, Clone, Node)]
pub struct Root {
    /// In library mode, the library path the user passed to us
    pub cdylib: Option<String>,
    pub modules: IndexMap<String, Module>,
}

#[derive(Debug, Clone, Node)]
pub struct Module {
    pub name: String,
    pub crate_name: String,
    /// contents of the `uniffi.toml` file for this module, if present
    pub config_toml: Option<String>,
    pub docstring: Option<String>,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
    pub ffi_definitions: IndexSet<FfiDefinition>,
    /// Checksum functions
    pub checksums: Vec<Checksum>,
    // FFI functions names for this module
    pub ffi_rustbuffer_alloc: RustFfiFunctionName,
    pub ffi_rustbuffer_from_bytes: RustFfiFunctionName,
    pub ffi_rustbuffer_free: RustFfiFunctionName,
    pub ffi_rustbuffer_reserve: RustFfiFunctionName,
    pub ffi_uniffi_contract_version: RustFfiFunctionName,
    // Correct contract version value
    pub correct_contract_version: String,
    // TypeNode for String.  Strings are used in error handling, this ensures that the FFI
    // converters for them are always defined and easy to lookup.
    pub string_type_node: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct Function {
    pub name: String,
    pub callable: Callable,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node)]
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

#[derive(Debug, Clone, Node)]
pub struct Constructor {
    pub name: String,
    pub callable: Callable,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node)]
pub struct Method {
    pub name: String,
    pub callable: Callable,
    pub is_async: bool,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

/// Common data from Function/Method/Constructor
#[derive(Debug, Clone, Node)]
pub struct Callable {
    pub name: String,
    pub is_async: bool,
    pub async_data: Option<AsyncData>,
    pub kind: CallableKind,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
    pub throws_type: ThrowsType,
    pub checksum: Option<u16>,
    pub ffi_func: RustFfiFunctionName,
}

#[derive(Debug, Clone, Node)]
pub enum CallableKind {
    /// Toplevel function
    Function,
    /// Interface/Trait interface method
    Method { interface_name: String },
    /// Interface constructor
    Constructor {
        interface_name: String,
        primary: bool,
    },
    /// Method inside a VTable or a CallbackInterface
    ///
    /// For trait interfaces this only applies to the Callables inside the `vtable.methods` field.
    /// Callables inside `Interface::methods` will still be `Callable::Method`.
    VTableMethod { trait_name: String },
}

#[derive(Debug, Clone, Node)]
pub struct ReturnType {
    pub ty: Option<TypeNode>,
}

#[derive(Debug, Clone, Node)]
pub struct ThrowsType {
    pub ty: Option<TypeNode>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct AsyncData {
    // FFI types for async Rust functions
    pub ffi_rust_future_poll: RustFfiFunctionName,
    pub ffi_rust_future_cancel: RustFfiFunctionName,
    pub ffi_rust_future_free: RustFfiFunctionName,
    pub ffi_rust_future_complete: RustFfiFunctionName,
    // FFI types for async foreign functions
    pub ffi_foreign_future_complete: FfiFunctionTypeName,
    pub ffi_foreign_future_result: FfiStructName,
}

#[derive(Debug, Clone, Node)]
pub struct Argument {
    pub name: String,
    pub ty: TypeNode,
    pub optional: bool,
    pub default: Option<DefaultValue>,
}

#[derive(Debug, Clone, Node)]
pub enum DefaultValue {
    Default(TypeNode),
    Literal(LiteralNode),
}

#[derive(Debug, Clone, Node)]
pub struct LiteralNode {
    #[node(wraps)]
    pub lit: Literal,
}

#[derive(Debug, Clone, Node)]
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
    Enum(String, TypeNode),
    EmptySequence,
    EmptyMap,
    None,
    Some { inner: Box<DefaultValue> },
}

// Represent the radix of integer literal values.
// We preserve the radix into the generated bindings for readability reasons.
#[derive(Debug, Clone, Node)]
pub enum Radix {
    Decimal = 10,
    Octal = 8,
    Hexadecimal = 16,
}

#[derive(Debug, Clone, Node)]
pub struct Record {
    pub name: String,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub enum FieldsKind {
    Unit,
    Named,
    Unnamed,
}

#[derive(Debug, Clone, Node)]
pub struct Field {
    pub name: String,
    pub ty: TypeNode,
    pub default: Option<DefaultValue>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node)]
pub enum EnumShape {
    Enum,
    Error { flat: bool },
}

#[derive(Debug, Clone, Node)]
pub struct Enum {
    pub name: String,
    /// Is this a "flat" enum -- one with no associated data
    pub is_flat: bool,
    pub shape: EnumShape,
    pub variants: Vec<Variant>,
    /// Enum discriminant type from the metadata.  This is the value specified in the source code.
    #[node(from(discr_type))]
    pub meta_discr_type: Option<TypeNode>,
    /// Enum discriminant type to use in generated code.  If the source code doesn't specify a
    /// type, this will be a sized integer type that's large enough to store all the discriminant
    /// values. We try to mimic what `rustc` does, but there's no guarantee that this will be
    /// exactly the same type.
    pub discr_type: TypeNode,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct Variant {
    pub name: String,
    /// Discriminant from the metadata.  This is the value specified in the source code.
    #[node(from(discr))]
    pub meta_discr: Option<LiteralNode>,
    /// Actual discriminant value.  When `meta_discr=None` this is determined using the Rust's rules
    /// for implicit discriminants:
    /// <https://doc.rust-lang.org/reference/items/enumerations.html#implicit-discriminants>
    pub discr: LiteralNode,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node)]
pub struct Interface {
    pub name: String,
    pub docstring: Option<String>,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
    pub uniffi_traits: Vec<UniffiTrait>,
    pub trait_impls: Vec<ObjectTraitImpl>,
    pub imp: ObjectImpl,
    pub self_type: TypeNode,
    pub vtable: Option<VTable>,
    pub ffi_func_clone: RustFfiFunctionName,
    pub ffi_func_free: RustFfiFunctionName,
}

#[derive(Debug, Clone, Node)]
pub struct CallbackInterface {
    pub name: String,
    pub docstring: Option<String>,
    pub vtable: VTable,
    pub methods: Vec<Method>,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct VTable {
    /// Vtable struct.  This has field for each callback interface method that stores a function
    /// pointer for that method.
    pub struct_type: FfiTypeNode,
    /// Name of the interface/callback interface that this vtable is for
    pub interface_name: String,
    /// Rust FFI function to initialize the vtable.
    ///
    /// Foreign code should call this function, passing it a pointer to the VTable struct.
    pub init_fn: RustFfiFunctionName,
    pub free_fn_type: FfiFunctionTypeName,
    pub methods: Vec<VTableMethod>,
}

/// Single method in a vtable
#[derive(Debug, Clone, Node)]
pub struct VTableMethod {
    pub callable: Callable,
    /// FfiType::Function type that corresponds to the method
    pub ffi_type: FfiTypeNode,
}

#[derive(Debug, Clone, Node)]
pub enum UniffiTrait {
    Debug { fmt: Method },
    Display { fmt: Method },
    Eq { eq: Method, ne: Method },
    Hash { hash: Method },
    Ord { cmp: Method },
}

#[derive(Debug, Clone, Node)]
pub struct ObjectTraitImpl {
    pub ty: TypeNode,
    pub trait_name: String,
    pub tr_module_name: Option<String>,
}

#[derive(Debug, Clone, Node)]
pub struct CustomType {
    pub name: String,
    pub builtin: TypeNode,
    pub docstring: Option<String>,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct OptionalType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct SequenceType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct MapType {
    pub key: TypeNode,
    pub value: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node)]
pub struct ExternalType {
    pub module_name: String,
    pub name: String,
    pub self_type: TypeNode,
}

/// Wrap `Type` so that we can add extra fields that are set for all variants.
#[derive(Debug, Clone, Node)]
pub struct TypeNode {
    #[node(wraps)]
    pub ty: Type,
    /// Unique UpperCamelCase name for the type
    ///
    /// This is used for a couple reasons:
    ///   - Defining classes to handle things related to the type.
    ///     Many bindings will create a `UniffiConverter[canonical_name]` class.
    ///   - Creating a unique key for a type
    pub canonical_name: String,
    pub is_used_as_error: bool,
    pub ffi_type: FfiTypeNode,
}

/// Like `TypeNode` but for FFI types.
///
/// This exists so that language bindings generators can add extra fields
#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiTypeNode {
    pub ty: FfiType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node)]
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
        module_name: String,
        name: String,
        imp: ObjectImpl,
    },
    Record {
        module_name: String,
        name: String,
    },
    Enum {
        module_name: String,
        name: String,
    },
    CallbackInterface {
        module_name: String,
        name: String,
    },
    Custom {
        module_name: String,
        name: String,
        builtin: Box<Type>,
    },
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub enum ObjectImpl {
    // A single Rust type
    Struct,
    // A trait that's can be implemented by Rust types
    Trait,
    // A trait + a callback interface -- can be implemented by both Rust and foreign types.
    CallbackTrait,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub enum FfiDefinition {
    /// FFI Function exported in the Rust library
    RustFunction(FfiFunction),
    /// FFI Function definition used in the interface, language, for example a callback interface method.
    FunctionType(FfiFunctionType),
    /// Struct definition used in the interface, for example a callback interface Vtable.
    Struct(FfiStruct),
}

/// Name of a FFI function from the Rust library
#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct RustFfiFunctionName(pub String);

/// Name of an FfiStruct
#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiStructName(pub String);

/// Name of an FfiFunctionType (i.e. a function pointer type)
#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiFunctionTypeName(pub String);

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiFunction {
    pub name: RustFfiFunctionName,
    pub is_async: bool,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
    pub kind: FfiFunctionKind,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub enum FfiFunctionKind {
    Scaffolding,
    ObjectClone,
    ObjectFree,
    RustFuturePoll,
    RustFutureComplete,
    RustFutureCancel,
    RustFutureFree,
    RustBufferFromBytes,
    RustBufferFree,
    RustBufferAlloc,
    RustBufferReserve,
    RustVtableInit,
    UniffiContractVersion,
    Checksum,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiFunctionType {
    pub name: FfiFunctionTypeName,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiReturnType {
    pub ty: Option<FfiTypeNode>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiStruct {
    pub name: FfiStructName,
    pub fields: Vec<FfiField>,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiField {
    pub name: String,
    pub ty: FfiTypeNode,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub struct FfiArgument {
    pub name: String,
    pub ty: FfiTypeNode,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub enum FfiType {
    // N.B. there are no booleans at this layer, since they cause problems for JNA.
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
    /// A byte buffer allocated by rust, and owned by whoever currently holds it.
    /// If you've got one of these, you must either call the appropriate rust function to free it
    /// or pass it to someone that will.
    ///
    /// For user-defined types like Record, Enum, CustomType, etc., the inner value will be the
    /// module name for that type.  This is needed for some languages, because each module
    /// defines a different RustBuffer type and using the wrong one will result in a type
    /// error.
    ///
    /// For builtin types like String, this is always None.  It's safe to assume that the
    /// RustBuffer is for the local module -- at least this has been true for all our languages
    /// so far, maybe we should revisit this.
    RustBuffer(Option<String>),
    /// A borrowed reference to some raw bytes owned by foreign language code.
    /// The provider of this reference must keep it alive for the duration of the receiving call.
    ForeignBytes,
    /// Value for a FfiDefinition::FunctionType.
    Function(FfiFunctionTypeName),
    /// Pointer to a FFI struct (e.g. a VTable).  The inner value matches one of the struct
    /// definitions in [crate::ComponentInterface::ffi_definitions].
    Struct(FfiStructName),
    /// Opaque 64-bit handle
    ///
    /// These are used to pass objects across the FFI.
    Handle(HandleKind),
    RustCallStatus,
    /// Const pointer to an FfiType.
    Reference(Box<FfiType>),
    /// Mutable pointer to an FfiType.
    MutReference(Box<FfiType>),
    /// Opaque pointer
    VoidPointer,
}

#[derive(Debug, Clone, Node, PartialEq, Eq, Hash)]
pub enum HandleKind {
    RustFuture,
    ForeignFuture,
    ForeignFutureCallbackData,
    // Interface, trait interface, or callback interface
    Interface {
        module_name: String,
        interface_name: String,
    },
}

#[derive(Debug, Clone, Node)]
pub struct Checksum {
    pub fn_name: RustFfiFunctionName,
    pub checksum: u16,
}

impl FfiDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::RustFunction(func) => &func.name.0,
            Self::FunctionType(func_type) => &func_type.name.0,
            Self::Struct(st) => &st.name.0,
        }
    }
}

impl From<FfiFunction> for FfiDefinition {
    fn from(func: FfiFunction) -> Self {
        Self::RustFunction(func)
    }
}

impl From<FfiFunctionType> for FfiDefinition {
    fn from(func_type: FfiFunctionType) -> Self {
        Self::FunctionType(func_type)
    }
}

impl From<FfiStruct> for FfiDefinition {
    fn from(st: FfiStruct) -> Self {
        Self::Struct(st)
    }
}

impl From<FfiType> for FfiTypeNode {
    fn from(ty: FfiType) -> Self {
        Self { ty }
    }
}

impl FfiArgument {
    pub fn new(name: impl Into<String>, ty: FfiType) -> Self {
        Self {
            name: name.into(),
            ty: FfiTypeNode { ty },
        }
    }
}

impl FfiField {
    pub fn new(name: impl Into<String>, ty: FfiType) -> Self {
        Self {
            name: name.into(),
            ty: FfiTypeNode { ty },
        }
    }
}

impl Callable {
    pub fn is_async(&self) -> bool {
        self.async_data.is_some()
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
}

impl ObjectImpl {
    pub fn has_callback_interface(&self) -> bool {
        matches!(self, Self::CallbackTrait)
    }
}
