/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Initial IR, this is the Metadata from uniffi_meta with some slight changes:
//!
//! * The Type/Literal enums are wrapped in TypeNode/LiteralNode structs. This allows for future pipeline passes to add fields.
//! * module_path is normalized to module_name (UDL and proc-macros determine module_path differently).

use indexmap::IndexMap;
pub mod pass;

use crate::ir::{ir, Node};

ir! {
    name: general;

    /// Initial IR, this stores the metadata and other data
    #[derive(Debug, Clone, Default, Node, PartialEq)]
    pub struct Root {
        /// In library mode, we get the name of the library file for free.
        pub cdylib: Option<String>,
        pub modules: IndexMap<String, Module>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Module {
        pub name: String,
        pub crate_name: String,
        pub docstring: Option<String>,
        pub functions: Vec<Function>,
        pub type_definitions: Vec<TypeDefinition>,
        pub ffi_definitions: Vec<FfiDefinition>,
        /// Checksum functions
        pub checksums: Vec<Checksum>,
        // FFI functions names for this module
        pub ffi_rustbuffer_alloc: RustFfiFunctionName,
        pub ffi_rustbuffer_from_bytes: RustFfiFunctionName,
        pub ffi_rustbuffer_free: RustFfiFunctionName,
        pub ffi_rustbuffer_reserve: RustFfiFunctionName,
        pub ffi_uniffi_contract_version: RustFfiFunctionName,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum TypeDefinition {
        Interface(Interface),
        CallbackInterface(CallbackInterface),
        Record(Record),
        Enum(Enum),
        Custom(CustomType),
        // Type that doesn't contain any other type
        Simple(TypeNode),
        // Compound types
        Optional(OptionalType),
        Sequence(SequenceType),
        Map(MapType),
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct NamespaceMetadata {
        pub crate_name: String,
        pub name: String,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Function {
        pub name: String,
        pub callable: Callable,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Constructor {
        pub name: String,
        pub self_name: String,
        pub callable: Callable,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Method {
        pub name: String,
        pub self_name: String,
        pub callable: Callable,
        pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
        pub docstring: Option<String>,
    }

    /// Common data from Function/Method/Constructor
    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Callable {
        pub name: String,
        pub async_data: Option<AsyncData>,
        pub kind: CallableKind,
        pub arguments: Vec<Argument>,
        pub return_type: ReturnType,
        pub throws_type: ThrowsType,
        pub checksum: Option<u16>,
        pub ffi_func: RustFfiFunctionName,
        #[pass_only]
        pub is_async: bool,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum CallableKind {
        Function,
        Method {
            interface_name: String,
        },
        VTableMethod {
            trait_name: String,
        },
        Constructor {
            interface_name: String,
            primary: bool,
        },
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct ReturnType {
        pub ty: Option<TypeNode>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct ThrowsType {
        pub ty: Option<TypeNode>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct AsyncData {
        pub ffi_rust_future_poll: FfiType,
        pub ffi_rust_future_cancel: FfiType,
        pub ffi_rust_future_free: FfiType,
        pub ffi_rust_future_complete: FfiType,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Argument {
        pub name: String,
        pub ty: TypeNode,
        pub by_ref: bool,
        pub optional: bool,
        pub default: Option<LiteralNode>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct LiteralNode {
        pub lit: Literal,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
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
        Some { inner: Box<Literal> },
    }

    // Represent the radix of integer literal values.
    // We preserve the radix into the generated bindings for readability reasons.
    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum Radix {
        Decimal = 10,
        Octal = 8,
        Hexadecimal = 16,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Record {
        pub name: String,
        pub remote: bool, // only used when generating scaffolding from UDL
        pub fields: Vec<Field>,
        pub docstring: Option<String>,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Field {
        pub name: String,
        pub ty: TypeNode,
        pub default: Option<LiteralNode>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum EnumShape {
        Enum,
        Error { flat: bool },
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Enum {
        pub name: String,
        pub shape: EnumShape,
        pub remote: bool,
        pub variants: Vec<Variant>,
        pub discr_type: Option<TypeNode>,
        pub non_exhaustive: bool,
        pub docstring: Option<String>,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Variant {
        pub name: String,
        pub discr: Option<LiteralNode>,
        pub fields: Vec<Field>,
        pub docstring: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Interface {
        pub name: String,
        pub constructors: Vec<Constructor>,
        pub methods: Vec<Method>,
        pub uniffi_traits: Vec<UniffiTrait>,
        pub trait_impls: Vec<ObjectTraitImpl>,
        pub remote: bool, // only used when generating scaffolding from UDL
        pub imp: ObjectImpl,
        pub docstring: Option<String>,
        pub self_type: TypeNode,
        pub vtable: Option<VTable>,
        pub ffi_func_clone: RustFfiFunctionName,
        pub ffi_func_free: RustFfiFunctionName,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct CallbackInterface {
        pub name: String,
        pub vtable: VTable,
        pub docstring: Option<String>,
        pub self_type: TypeNode,
        #[pass_only]
        // Methods get moved to `self.vtable` after the pass
        pub methods: Vec<Method>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct VTable {
        /// Vtable struct.  This has field for each callback interface method that stores a function
        /// pointer for that method.
        pub struct_type: FfiType,
        /// Rust FFI function to initialize the vtable.
        ///
        /// Foreign code should call this function, passing it a pointer to the VTable struct.
        pub init_fn: RustFfiFunctionName,
        pub methods: Vec<VTableMethod>,
    }

    /// Single method in a vtable
    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct VTableMethod {
        pub callable: Callable,
        /// FfiType::Function type that corresponds to the method
        pub ffi_type: FfiType,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum UniffiTrait {
        Debug {
            fmt: Method,
        },
        Display {
            fmt: Method,
        },
        Eq {
            eq: Method,
            ne: Method,
        },
        Hash {
            hash: Method,
        },
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct ObjectTraitImpl {
        pub ty: TypeNode,
        pub trait_name: String,
        pub tr_module_name: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct CustomType {
        pub name: String,
        pub builtin: TypeNode,
        pub docstring: Option<String>,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct OptionalType {
        pub inner: TypeNode,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct SequenceType {
        pub inner: TypeNode,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct MapType {
        pub key: TypeNode,
        pub value: TypeNode,
        pub self_type: TypeNode,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct TypeNode {
        pub ty: Type,
        pub is_used_as_error: bool,
        pub ffi_type: FfiType,
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
        Interface {
            // The module path to the object
            module_name: String,
            // The name in the "type universe"
            name: String,
            // How the object is implemented.
            imp: ObjectImpl,
        },
        // Types defined in the component API, each of which has a string name.
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
        // Custom type on the scaffolding side
        Custom {
            module_name: String,
            name: String,
            builtin: Box<Type>,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Node)]
    pub enum ObjectImpl {
        // A single Rust type
        Struct,
        // A trait that's can be implemented by Rust types
        Trait,
        // A trait + a callback interface -- can be implemented by both Rust and foreign types.
        CallbackTrait,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub enum FfiDefinition {
        /// FFI Function exported in the Rust library
        RustFunction(FfiFunction),
        /// FFI Function definition used in the interface, language, for example a callback interface method.
        FunctionType(FfiFunctionType),
        /// Struct definition used in the interface, for example a callback interface Vtable.
        Struct(FfiStruct),
    }

    /// Name of a FFI function from the Rust library
    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct RustFfiFunctionName(pub String);

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiFunction {
        pub name: String,
        pub is_async: bool,
        pub arguments: Vec<FfiArgument>,
        pub return_type: FfiReturnType,
        pub has_rust_call_status_arg: bool,
        pub kind: FfiFunctionKind,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
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

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiFunctionType {
        pub name: String,
        pub arguments: Vec<FfiArgument>,
        pub return_type: FfiReturnType,
        pub has_rust_call_status_arg: bool,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiReturnType {
        pub ty: Option<FfiType>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiStruct {
        pub name: String,
        pub fields: Vec<FfiField>,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiField {
        pub name: String,
        pub ty: FfiType,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct FfiArgument {
        pub name: String,
        pub ty: FfiType,
    }

    #[derive(Debug, Clone, PartialEq, Node)]
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
        /// A `*const c_void` pointer to a rust-owned `Arc<T>`.
        /// If you've got one of these, you must call the appropriate rust function to free it.
        /// The templates will generate a unique `free` function for each T.
        /// The inner string references the name of the `T` type.
        RustArcPtr {
            module_name: String,
            object_name: String,
        },
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
        Function(String),
        /// Pointer to a FFI struct (e.g. a VTable).  The inner value matches one of the struct
        /// definitions in [crate::ComponentInterface::ffi_definitions].
        Struct(String),
        /// Opaque 64-bit handle
        ///
        /// These are used to pass objects across the FFI.
        Handle,
        RustCallStatus,
        /// Const pointer to an FfiType.
        Reference(Box<FfiType>),
        /// Mutable pointer to an FfiType.
        MutReference(Box<FfiType>),
        /// Opaque pointer
        VoidPointer,
    }


    #[derive(Debug, Clone, PartialEq, Node)]
    pub struct Checksum {
        pub fn_name: String,
        pub checksum: u16,
    }

    impl Type {
        /// Unique UpperCamelCase name for the type
        ///
        /// This is used for a couple reasons:
        ///   - Defining classes to handle things related to the type.  Many bindings will create a
        ///   `UniffiConverter[canonical_name]`  class.
        ///   - Creating a unique key for a type
        pub fn canonical_name(&self) -> String {
            match self {
                Type::UInt8 => "UInt8".to_string(),
                Type::Int8 => "Int8".to_string(),
                Type::UInt16 => "UInt16".to_string(),
                Type::Int16 => "Int16".to_string(),
                Type::UInt32 => "UInt32".to_string(),
                Type::Int32 => "Int32".to_string(),
                Type::UInt64 => "UInt64".to_string(),
                Type::Int64 => "Int64".to_string(),
                Type::Float32 => "Float32".to_string(),
                Type::Float64 => "Float64".to_string(),
                Type::Boolean => "Boolean".to_string(),
                Type::String => "String".to_string(),
                Type::Bytes => "Bytes".to_string(),
                Type::Timestamp => "Timestamp".to_string(),
                Type::Duration => "Duration".to_string(),
                Type::Interface { name, .. }
                | Type::Record { name, .. }
                | Type::Enum { name, .. }
                | Type::CallbackInterface { name, .. }
                | Type::Custom { name, .. } => format!("Type{name}"),
                Type::Optional { inner_type } => {
                    format!("Optional{}", inner_type.canonical_name())
                }
                Type::Sequence { inner_type } => {
                    format!("Sequence{}", inner_type.canonical_name())
                }
                // Note: this is currently guaranteed to be unique because keys can only be primitive
                // types.  If we allowed user-defined types, there would be potential collisions.  For
                // example "MapTypeFooTypeTypeBar" could be "Foo" -> "TypeBar" or "FooType" -> "Bar".
                Type::Map {
                    key_type,
                    value_type,
                } => format!(
                    "Map{}{}",
                    key_type.canonical_name(),
                    value_type.canonical_name(),
                ),
                #[allow(unreachable_patterns)]
                _ => panic!("Invalid type in Type::canonical_name: {self:?}"),
            }
        }
    }

    impl FfiDefinition {
        pub fn name(&self) -> &str {
            match self {
                Self::RustFunction(func) => &func.name,
                Self::FunctionType(func_type) => &func_type.name,
                Self::Struct(st) => &st.name,
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

    impl Callable {
        pub fn is_async(&self) -> bool {
            self.async_data.is_some()
        }

        pub fn ffi_return_type(&self) -> Option<&FfiType> {
            self.return_type.ty.as_ref().map(|ty| &ty.ffi_type)
        }
    }

    impl ObjectImpl {
        pub fn has_callback_interface(&self) -> bool {
            matches!(self, Self::CallbackTrait)
        }
    }
}
