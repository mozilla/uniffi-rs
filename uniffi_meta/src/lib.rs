/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::BTreeMap, hash::Hasher};
pub use uniffi_internal_macros::{Checksum, Node};

mod ffi_names;
pub use ffi_names::*;

mod group;
pub use group::{create_metadata_groups, group_metadata, MetadataGroup};

mod reader;
pub use reader::{read_metadata, read_metadata_type};

mod types;
pub use types::{AsType, ObjectImpl, Type, TypeIterator};

mod metadata;

// This needs to match the minor version of the `uniffi` crate.  See
// `docs/uniffi-versioning.md` for details.
//
// Once we get to 1.0, then we'll need to update the scheme to something like 100 + major_version
pub const UNIFFI_CONTRACT_VERSION: u32 = 30;

/// Similar to std::hash::Hash.
///
/// Implementations of this trait are expected to update the hasher state in
/// the same way across platforms. #[derive(Checksum)] will do the right thing.
pub trait Checksum {
    fn checksum<H: Hasher>(&self, state: &mut H);
}

impl Checksum for bool {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self as u8);
    }
}

impl Checksum for u64 {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_le_bytes());
    }
}

impl Checksum for i64 {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_le_bytes());
    }
}

impl<T: Checksum> Checksum for Box<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

impl<T: Checksum> Checksum for [T] {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&(self.len() as u64).to_le_bytes());
        for item in self {
            Checksum::checksum(item, state);
        }
    }
}

impl<T: Checksum> Checksum for Vec<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        Checksum::checksum(&**self, state);
    }
}

impl<K: Checksum, V: Checksum> Checksum for BTreeMap<K, V> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(&(self.len() as u64).to_le_bytes());
        for (key, value) in self {
            Checksum::checksum(key, state);
            Checksum::checksum(value, state);
        }
    }
}

impl<T: Checksum> Checksum for Option<T> {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        match self {
            None => state.write(&0u64.to_le_bytes()),
            Some(value) => {
                state.write(&1u64.to_le_bytes());
                Checksum::checksum(value, state)
            }
        }
    }
}

impl Checksum for str {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
        state.write_u8(0xff);
    }
}

impl Checksum for String {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

impl Checksum for &str {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        (**self).checksum(state)
    }
}

// The namespace of a Component interface.
//
// This is used to match up the macro metadata with the UDL items.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct NamespaceMetadata {
    pub crate_name: String,
    pub name: String,
}

// UDL file included with `include_scaffolding!()`
//
// This is to find the UDL files in library mode generation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct UdlFile {
    // The module path specified when the UDL file was parsed.
    pub module_path: String,
    pub namespace: String,
    // the base filename of the udl file - no path, no extension.
    pub file_stub: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct FnMetadata {
    pub module_path: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

impl FnMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        fn_symbol_name(&self.module_path, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        fn_checksum_symbol_name(&self.module_path, &self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct ConstructorMetadata {
    pub module_path: String,
    pub self_name: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub throws: Option<Type>,
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

impl ConstructorMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        constructor_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        constructor_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn is_primary(&self) -> bool {
        self.name == "new"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct MethodMetadata {
    pub module_path: String,
    pub self_name: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

impl MethodMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        method_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        method_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct TraitMethodMetadata {
    pub module_path: String,
    pub trait_name: String,
    // Note: the position of `index` is important since it causes callback interface methods to be
    // ordered correctly in MetadataGroup.items
    pub index: u32,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub takes_self_by_arc: bool, // unused except by rust udl bindgen.
    pub checksum: Option<u16>,
    pub docstring: Option<String>,
}

impl TraitMethodMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        method_symbol_name(&self.module_path, &self.trait_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        method_checksum_symbol_name(&self.module_path, &self.trait_name, &self.name)
    }
}

impl From<TraitMethodMetadata> for MethodMetadata {
    fn from(meta: TraitMethodMetadata) -> Self {
        MethodMetadata {
            module_path: meta.module_path,
            self_name: meta.trait_name,
            name: meta.name,
            is_async: meta.is_async,
            inputs: meta.inputs,
            return_type: meta.return_type,
            throws: meta.throws,
            takes_self_by_arc: meta.takes_self_by_arc,
            checksum: meta.checksum,
            docstring: meta.docstring,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct FnParamMetadata {
    pub name: String,
    pub ty: Type,
    pub by_ref: bool,
    pub optional: bool,
    pub default: Option<DefaultValueMetadata>,
}

impl FnParamMetadata {
    pub fn simple(name: &str, ty: Type) -> Self {
        Self {
            name: name.to_string(),
            ty,
            by_ref: false,
            optional: false,
            default: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Checksum, Node)]
pub enum LiteralMetadata {
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
    Some { inner: Box<DefaultValueMetadata> },
}

impl LiteralMetadata {
    pub fn new_uint(v: u64) -> Self {
        LiteralMetadata::UInt(v, Radix::Decimal, Type::UInt64)
    }
    pub fn new_int(v: i64) -> Self {
        LiteralMetadata::Int(v, Radix::Decimal, Type::Int64)
    }
}

// Represent the radix of integer literal values.
// We preserve the radix into the generated bindings for readability reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Checksum, Node)]
pub enum Radix {
    Decimal = 10,
    Octal = 8,
    Hexadecimal = 16,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Checksum, Node)]
pub enum DefaultValueMetadata {
    // unspecified default value
    Default,
    // an explicit literal value.
    Literal(LiteralMetadata),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct RecordMetadata {
    pub module_path: String,
    pub name: String,
    pub remote: bool, // only used when generating scaffolding from UDL
    pub fields: Vec<FieldMetadata>,
    pub docstring: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct FieldMetadata {
    pub name: String,
    pub ty: Type,
    pub default: Option<DefaultValueMetadata>,
    pub docstring: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Checksum, Node)]
pub enum EnumShape {
    Enum,
    Error { flat: bool },
}

impl EnumShape {
    pub fn as_u8(&self) -> u8 {
        match self {
            EnumShape::Enum => 0,
            EnumShape::Error { flat: false } => 1,
            EnumShape::Error { flat: true } => 2,
        }
    }

    pub fn from(v: u8) -> anyhow::Result<Self> {
        Ok(match v {
            0 => EnumShape::Enum,
            1 => EnumShape::Error { flat: false },
            2 => EnumShape::Error { flat: true },
            _ => anyhow::bail!("invalid enum shape discriminant {v}"),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct EnumMetadata {
    pub module_path: String,
    pub name: String,
    pub shape: EnumShape,
    pub remote: bool, // only used when generating scaffolding from UDL
    pub variants: Vec<VariantMetadata>,
    pub discr_type: Option<Type>,
    pub non_exhaustive: bool,
    pub docstring: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct VariantMetadata {
    pub name: String,
    pub discr: Option<LiteralMetadata>,
    pub fields: Vec<FieldMetadata>,
    pub docstring: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct ObjectMetadata {
    pub module_path: String,
    pub name: String,
    pub remote: bool, // only used when generating scaffolding from UDL
    pub imp: types::ObjectImpl,
    pub docstring: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct CallbackInterfaceMetadata {
    pub module_path: String,
    pub name: String,
    pub docstring: Option<String>,
}

impl ObjectMetadata {
    /// FFI symbol name for the `clone` function for this object.
    ///
    /// This function is used to increment the reference count before lowering an object to pass
    /// back to Rust.
    pub fn clone_ffi_symbol_name(&self) -> String {
        clone_fn_symbol_name(&self.module_path, &self.name)
    }

    /// FFI symbol name for the `free` function for this object.
    ///
    /// This function is used to free the memory used by this object.
    pub fn free_ffi_symbol_name(&self) -> String {
        free_fn_symbol_name(&self.module_path, &self.name)
    }
}

/// The list of "builtin" traits we support generating helper methods for.
/// Some interesting overlap with ObjectTraitImplMetadata, but quite different
/// implementations for now.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub enum UniffiTraitMetadata {
    Debug {
        fmt: MethodMetadata,
    },
    Display {
        fmt: MethodMetadata,
    },
    Eq {
        eq: MethodMetadata,
        ne: MethodMetadata,
    },
    Hash {
        hash: MethodMetadata,
    },
    Ord {
        cmp: MethodMetadata,
    },
}

impl UniffiTraitMetadata {
    fn module_path(&self) -> &str {
        &match self {
            UniffiTraitMetadata::Debug { fmt } => fmt,
            UniffiTraitMetadata::Display { fmt } => fmt,
            UniffiTraitMetadata::Eq { eq, .. } => eq,
            UniffiTraitMetadata::Hash { hash } => hash,
            UniffiTraitMetadata::Ord { cmp } => cmp,
        }
        .module_path
    }

    pub fn self_name(&self) -> &str {
        &match self {
            UniffiTraitMetadata::Debug { fmt } => fmt,
            UniffiTraitMetadata::Display { fmt } => fmt,
            UniffiTraitMetadata::Eq { eq, .. } => eq,
            UniffiTraitMetadata::Hash { hash } => hash,
            UniffiTraitMetadata::Ord { cmp } => cmp,
        }
        .self_name
    }

    pub fn name(&self) -> &str {
        &match self {
            UniffiTraitMetadata::Debug { fmt } => fmt,
            UniffiTraitMetadata::Display { fmt } => fmt,
            UniffiTraitMetadata::Eq { eq, .. } => eq,
            UniffiTraitMetadata::Hash { hash } => hash,
            UniffiTraitMetadata::Ord { cmp } => cmp,
        }
        .name
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Hash, Node)]
pub enum UniffiTraitDiscriminants {
    Debug,
    Display,
    Eq,
    Hash,
    Ord,
}

impl UniffiTraitDiscriminants {
    pub fn from(v: u8) -> anyhow::Result<Self> {
        Ok(match v {
            0 => UniffiTraitDiscriminants::Debug,
            1 => UniffiTraitDiscriminants::Display,
            2 => UniffiTraitDiscriminants::Eq,
            3 => UniffiTraitDiscriminants::Hash,
            4 => UniffiTraitDiscriminants::Ord,
            _ => anyhow::bail!("invalid trait discriminant {v}"),
        })
    }
}

/// This notes that a type implements a Trait.
/// eg, an `impl Tr for Ob` block. Not many types will support this.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct ObjectTraitImplMetadata {
    pub ty: Type,
    pub trait_name: String,
    pub tr_module_path: Option<String>,
}

impl Checksum for ObjectTraitImplMetadata {
    fn checksum<H: Hasher>(&self, state: &mut H) {
        Checksum::checksum(&self.ty, state);
        Checksum::checksum(&self.trait_name, state);
        Checksum::checksum(&self.tr_module_path, state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub struct CustomTypeMetadata {
    pub module_path: String,
    pub name: String,
    pub builtin: Type,
    pub docstring: Option<String>,
}

/// Returns the last 16 bits of the value's hash as computed with [`SipHasher13`].
///
/// This is used as a safeguard against different UniFFI versions being used for scaffolding and
/// bindings generation.
pub fn checksum<T: Checksum>(val: &T) -> u16 {
    let mut hasher = siphasher::sip::SipHasher13::new();
    val.checksum(&mut hasher);
    (hasher.finish() & 0x000000000000FFFF) as u16
}

/// Enum covering all the possible metadata types
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Node)]
pub enum Metadata {
    Namespace(NamespaceMetadata),
    UdlFile(UdlFile),
    Func(FnMetadata),
    Object(ObjectMetadata),
    CallbackInterface(CallbackInterfaceMetadata),
    Record(RecordMetadata),
    Enum(EnumMetadata),
    Constructor(ConstructorMetadata),
    Method(MethodMetadata),
    TraitMethod(TraitMethodMetadata),
    CustomType(CustomTypeMetadata),
    UniffiTrait(UniffiTraitMetadata),
    ObjectTraitImpl(ObjectTraitImplMetadata),
}

impl Metadata {
    pub fn read(data: &[u8]) -> anyhow::Result<Self> {
        read_metadata(data)
    }

    pub fn module_path(&self) -> &str {
        match self {
            Metadata::Namespace(meta) => &meta.crate_name,
            Metadata::UdlFile(meta) => &meta.module_path,
            Metadata::Func(meta) => &meta.module_path,
            Metadata::Constructor(meta) => &meta.module_path,
            Metadata::Method(meta) => &meta.module_path,
            Metadata::Record(meta) => &meta.module_path,
            Metadata::Enum(meta) => &meta.module_path,
            Metadata::Object(meta) => &meta.module_path,
            Metadata::CallbackInterface(meta) => &meta.module_path,
            Metadata::TraitMethod(meta) => &meta.module_path,
            Metadata::CustomType(meta) => &meta.module_path,
            Metadata::UniffiTrait(meta) => meta.module_path(),
            Metadata::ObjectTraitImpl(t) => t.ty.module_path().expect("type has no module"),
        }
    }
}

impl From<NamespaceMetadata> for Metadata {
    fn from(value: NamespaceMetadata) -> Metadata {
        Self::Namespace(value)
    }
}

impl From<UdlFile> for Metadata {
    fn from(value: UdlFile) -> Metadata {
        Self::UdlFile(value)
    }
}

impl From<FnMetadata> for Metadata {
    fn from(value: FnMetadata) -> Metadata {
        Self::Func(value)
    }
}

impl From<ConstructorMetadata> for Metadata {
    fn from(c: ConstructorMetadata) -> Self {
        Self::Constructor(c)
    }
}

impl From<MethodMetadata> for Metadata {
    fn from(m: MethodMetadata) -> Self {
        Self::Method(m)
    }
}

impl From<RecordMetadata> for Metadata {
    fn from(r: RecordMetadata) -> Self {
        Self::Record(r)
    }
}

impl From<EnumMetadata> for Metadata {
    fn from(e: EnumMetadata) -> Self {
        Self::Enum(e)
    }
}

impl From<ObjectMetadata> for Metadata {
    fn from(v: ObjectMetadata) -> Self {
        Self::Object(v)
    }
}

impl From<CallbackInterfaceMetadata> for Metadata {
    fn from(v: CallbackInterfaceMetadata) -> Self {
        Self::CallbackInterface(v)
    }
}

impl From<TraitMethodMetadata> for Metadata {
    fn from(v: TraitMethodMetadata) -> Self {
        Self::TraitMethod(v)
    }
}

impl From<CustomTypeMetadata> for Metadata {
    fn from(v: CustomTypeMetadata) -> Self {
        Self::CustomType(v)
    }
}

impl From<UniffiTraitMetadata> for Metadata {
    fn from(v: UniffiTraitMetadata) -> Self {
        Self::UniffiTrait(v)
    }
}

impl From<ObjectTraitImplMetadata> for Metadata {
    fn from(t: ObjectTraitImplMetadata) -> Self {
        Self::ObjectTraitImpl(t)
    }
}
