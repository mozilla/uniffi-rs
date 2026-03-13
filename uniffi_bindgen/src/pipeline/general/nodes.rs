/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

use_prev_node!(initial::EnumShape);
use_prev_node!(initial::ObjectImpl);
use_prev_node!(initial::Radix);
use_prev_node!(initial::Type, types::map_type);

/// Initial IR, this stores the metadata and other data
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Root))]
#[map_node(update_context(context.update_from_root(&self)?))]
pub struct Root {
    /// In library mode, the library path the user passed to us
    pub cdylib: Option<String>,
    pub namespaces: IndexMap<String, Namespace>,
}

/// A Namespace is a crate which exposes a uniffi api.
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Namespace))]
#[map_node(namespaces::map_namespace)]
pub struct Namespace {
    pub name: String,
    pub crate_name: String,
    /// contents of the `uniffi.toml` file for this namespace, if present
    pub config_toml: Option<String>,
    pub docstring: Option<String>,
    pub functions: Vec<Function>,
    pub type_definitions: Vec<TypeDefinition>,
    pub ffi_definitions: IndexSet<FfiDefinition>,
    /// Checksum functions
    pub checksums: Vec<Checksum>,
    // FFI functions names in this namespace
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

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Function))]
pub struct Function {
    #[map_node(callable::function_callable(&self, context)?)]
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::TypeDefinition))]
pub enum TypeDefinition {
    Interface(Interface),
    CallbackInterface(CallbackInterface),
    Record(Record),
    Enum(Enum),
    Custom(CustomType),
    /// Type that doesn't contain any other type
    #[map_node(added)]
    Simple(TypeNode),
    /// Compound types
    #[map_node(added)]
    Optional(OptionalType),
    #[map_node(added)]
    Sequence(SequenceType),
    #[map_node(added)]
    Map(MapType),
    /// User types that are defined in another crate
    #[map_node(added)]
    External(ExternalType),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Constructor))]
pub struct Constructor {
    #[map_node(callable::constructor_callable(&self, context)?)]
    pub callable: Callable,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Method))]
pub struct Method {
    #[map_node(callable::method_callable(&self, context)?)]
    pub callable: Callable,
    pub docstring: Option<String>,
}

/// Common data from Function/Method/Constructor
#[derive(Debug, Clone, Node, MapNode)]
pub struct Callable {
    pub name: String,
    pub async_data: Option<AsyncData>,
    pub kind: CallableKind,
    pub arguments: Vec<Argument>,
    pub return_type: ReturnType,
    pub throws_type: ThrowsType,
    pub checksum: Option<u16>,
    pub ffi_func: RustFfiFunctionName,
}

#[derive(Debug, Clone, Node, MapNode)]
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

#[derive(Debug, Clone, Node, MapNode)]
pub struct ReturnType {
    pub ty: Option<TypeNode>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct ThrowsType {
    pub ty: Option<TypeNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
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

/// Default value for a field/argument
///
/// This sets the arg/field type in the case where the user just specified `default`.
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::DefaultValue))]
#[map_node(default::map_default_value)]
pub enum DefaultValue {
    Literal(Literal),
    Default(TypeNode),
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Literal))]
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

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Record))]
#[map_node(update_context(context.update_from_record(&self)?))]
pub struct Record {
    #[map_node(records::fields_kind(&self.fields))]
    pub fields_kind: FieldsKind,
    #[map_node(context.self_type()?)]
    pub self_type: TypeNode,
    #[map_node(self.name.clone())]
    pub orig_name: String,
    #[map_node(rename::type_(&context.namespace_name()?, self.name, context)?)]
    pub name: String,
    #[map_node(from(uniffi_traits))]
    pub uniffi_trait_methods: UniffiTraitMethods,
    pub fields: Vec<Field>,
    #[map_node(objects::constructors(self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(objects::methods(self.methods, context)?)]
    pub methods: Vec<Method>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub enum FieldsKind {
    Unit,
    Named,
    Unnamed,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Field))]
#[map_node(update_context(context.update_from_field(&self)?))]
pub struct Field {
    #[map_node(rename::field(self.name, context)?)]
    pub name: String,
    pub ty: TypeNode,
    pub default: Option<DefaultValue>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Enum))]
#[map_node(update_context(context.update_from_enum(&self)?))]
pub struct Enum {
    /// Is this a "flat" enum -- one with no associated data
    #[map_node(enums::is_flat(&self))]
    pub is_flat: bool,
    #[map_node(context.self_type()?)]
    pub self_type: TypeNode,
    /// type, this will be a sized integer type that's large enough to store all the discriminant
    /// values. We try to mimic what `rustc` does, but there's no guarantee that this will be
    /// exactly the same type.
    #[map_node(enums::discr_type(&self, context)?)]
    pub discr_type: TypeNode,
    #[map_node(enums::map_variants(&self.discr_type, self.variants, context)?)]
    pub variants: Vec<Variant>,
    #[map_node(self.name.clone())]
    pub orig_name: String,
    #[map_node(rename::type_(&context.namespace_name()?, self.name, context)?)]
    pub name: String,
    #[map_node(from(uniffi_traits))]
    pub uniffi_trait_methods: UniffiTraitMethods,
    /// Enum discriminant type to use in generated code.  If the source code doesn't specify a
    pub shape: EnumShape,
    #[map_node(objects::constructors(self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(objects::methods(self.methods, context)?)]
    pub methods: Vec<Method>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct Variant {
    pub name: String,
    /// Actual discriminant value.  If the source code doesn't specify a discriminant,
    /// this is determined using the Rust's rules for implicit discriminants:
    /// <https://doc.rust-lang.org/reference/items/enumerations.html#implicit-discriminants>
    pub discr: Literal,
    pub fields_kind: FieldsKind,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::Interface))]
#[map_node(update_context(context.update_from_interface(&self)?))]
pub struct Interface {
    #[map_node(callback_interfaces::vtable_for_interface(&self, context)?)]
    pub vtable: Option<VTable>,
    #[map_node(context.self_type()?)]
    pub self_type: TypeNode,
    #[map_node(objects::ffi_clone_name(&self.name, context)?)]
    pub ffi_func_clone: RustFfiFunctionName,
    #[map_node(objects::ffi_free_name(&self.name, context)?)]
    pub ffi_func_free: RustFfiFunctionName,
    #[map_node(self.name.clone())]
    pub orig_name: String,
    #[map_node(rename::type_(&context.namespace_name()?, self.name, context)?)]
    pub name: String,
    // This `map_node` works because we've implemented a map from Vec<UniffiTrait> -> UniffiTraitMethods
    #[map_node(from(uniffi_traits))]
    pub uniffi_trait_methods: UniffiTraitMethods,
    pub docstring: Option<String>,
    #[map_node(objects::constructors(self.constructors, context)?)]
    pub constructors: Vec<Constructor>,
    #[map_node(objects::methods(self.methods, context)?)]
    pub methods: Vec<Method>,
    pub trait_impls: Vec<ObjectTraitImpl>,
    pub imp: ObjectImpl,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::CallbackInterface))]
#[map_node(update_context(context.update_from_callback_interface(&self)?))]
pub struct CallbackInterface {
    #[map_node(callback_interfaces::vtable(&self.methods, context)?)]
    pub vtable: VTable,
    #[map_node(context.self_type()?)]
    pub self_type: TypeNode,
    #[map_node(self.name.clone())]
    pub orig_name: String,
    #[map_node(rename::type_(&context.namespace_name()?, self.name, context)?)]
    pub name: String,
    pub docstring: Option<String>,
    #[map_node(objects::methods(self.methods, context)?)]
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct VTable {
    /// Vtable struct.  This has field for each callback interface method that stores a function
    /// pointer for that method.
    pub struct_type: FfiType,
    /// Name of the interface/callback interface that this vtable is for
    pub interface_name: String,
    /// Rust FFI function to initialize the vtable.
    ///
    /// Foreign code should call this function, passing it a pointer to the VTable struct.
    pub init_fn: RustFfiFunctionName,
    pub clone_fn_type: FfiFunctionTypeName,
    pub free_fn_type: FfiFunctionTypeName,
    pub methods: Vec<VTableMethod>,
}

/// Single method in a vtable
#[derive(Debug, Clone, Node, MapNode)]
pub struct VTableMethod {
    pub callable: Callable,
    /// FfiType::Function type that corresponds to the method
    pub ffi_type: FfiType,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::ObjectTraitImpl))]
pub struct ObjectTraitImpl {
    pub ty: TypeNode,
    pub trait_ty: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(initial::CustomType))]
#[map_node(update_context(context.update_from_custom_type(&self)?))]
pub struct CustomType {
    #[map_node(context.self_type()?)]
    pub self_type: TypeNode,
    #[map_node(self.name.clone())]
    pub orig_name: String,
    #[map_node(rename::type_(&context.namespace_name()?, self.name, context)?)]
    pub name: String,
    pub builtin: TypeNode,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct OptionalType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct SequenceType {
    pub inner: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct MapType {
    pub key: TypeNode,
    pub value: TypeNode,
    pub self_type: TypeNode,
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct ExternalType {
    pub namespace: String,
    pub name: String,
    pub self_type: TypeNode,
}

/// Wrap `Type` so that we can add extra fields that are set for all variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
#[map_node(from(Type))]
pub struct TypeNode {
    /// Unique UpperCamelCase name for the type
    ///
    /// This is used for a couple reasons:
    ///   - Defining classes to handle things related to the type.
    ///     Many bindings will create a `UniffiConverter[canonical_name]` class.
    ///   - Creating a unique key for a type
    #[map_node(types::canonical_name(&self))]
    pub canonical_name: String,
    #[map_node(context.type_is_used_as_error(&self))]
    pub is_used_as_error: bool,
    #[map_node(ffi_types::ffi_type(&self, context)?)]
    pub ffi_type: FfiType,
    #[map_node(self.map_node(context)?)]
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub enum FfiDefinition {
    /// FFI Function exported in the Rust library
    RustFunction(FfiFunction),
    /// FFI Function definition used in the interface, language, for example a callback interface method.
    FunctionType(FfiFunctionType),
    /// Struct definition used in the interface, for example a callback interface Vtable.
    Struct(FfiStruct),
}

/// Name of a FFI function from the Rust library
#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct RustFfiFunctionName(pub String);

/// Name of an FfiStruct
#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiStructName(pub String);

/// Name of an FfiFunctionType (i.e. a function pointer type)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiFunctionTypeName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiFunction {
    pub name: RustFfiFunctionName,
    pub async_data: Option<AsyncData>,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
    pub kind: FfiFunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiFunctionType {
    pub name: FfiFunctionTypeName,
    pub arguments: Vec<FfiArgument>,
    pub return_type: FfiReturnType,
    pub has_rust_call_status_arg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiReturnType {
    pub ty: Option<FfiType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiStruct {
    pub name: FfiStructName,
    pub fields: Vec<FfiField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiField {
    pub name: String,
    pub ty: FfiType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub struct FfiArgument {
    pub name: String,
    pub ty: FfiType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
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
    /// namespace name for that type.  This is needed for some languages, because each module
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
pub enum HandleKind {
    RustFuture,
    ForeignFuture,
    ForeignFutureCallbackData,
    // Interface, trait interface, or callback interface
    StructInterface {
        namespace: String,
        interface_name: String,
    },
    TraitInterface {
        namespace: String,
        interface_name: String,
    },
}

#[derive(Debug, Clone, Node, MapNode)]
pub struct Checksum {
    pub fn_name: RustFfiFunctionName,
    pub checksum: u16,
}

/// Set of methods for builtin traits
///
/// This gets mapped from a list of `initial::UniffiTrait` items.
#[derive(Default, Debug, Clone, Node, MapNode)]
#[map_node(from(Vec<initial::UniffiTrait>))]
#[map_node(uniffi_traits::map_trait_vec)]
pub struct UniffiTraitMethods {
    pub debug_fmt: Option<Method>,
    pub display_fmt: Option<Method>,
    pub eq_eq: Option<Method>,
    pub eq_ne: Option<Method>,
    pub hash_hash: Option<Method>,
    pub ord_cmp: Option<Method>,
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

impl FfiArgument {
    pub fn new(name: impl Into<String>, ty: FfiType) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

impl FfiField {
    pub fn new(name: impl Into<String>, ty: FfiType) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

impl Callable {
    pub fn is_async(&self) -> bool {
        self.async_data.is_some()
    }

    pub fn is_primary_constructor(&self) -> bool {
        matches!(self.kind, CallableKind::Constructor { primary: true, .. })
    }
}
