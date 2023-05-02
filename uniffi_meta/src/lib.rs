/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::BTreeMap, hash::Hasher};
pub use uniffi_checksum_derive::Checksum;

use serde::{Deserialize, Serialize};

mod ffi_names;
pub use ffi_names::*;

mod reader;
pub use reader::{read_metadata, read_metadata_type};

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
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NamespaceMetadata {
    pub crate_name: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct FnMetadata {
    pub module_path: String,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: u16,
}

impl FnMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        fn_symbol_name(&self.module_path, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        fn_checksum_symbol_name(&self.module_path, &self.name)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ConstructorMetadata {
    pub module_path: String,
    pub self_name: String,
    pub name: String,
    pub inputs: Vec<FnParamMetadata>,
    pub throws: Option<Type>,
    pub checksum: u16,
}

impl ConstructorMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        constructor_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        constructor_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MethodMetadata {
    pub module_path: String,
    pub self_name: String,
    pub self_is_trait: bool,
    pub name: String,
    pub is_async: bool,
    pub inputs: Vec<FnParamMetadata>,
    pub return_type: Option<Type>,
    pub throws: Option<Type>,
    pub checksum: u16,
}

impl MethodMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        method_symbol_name(&self.module_path, &self.self_name, &self.name)
    }

    pub fn checksum_symbol_name(&self) -> String {
        method_checksum_symbol_name(&self.module_path, &self.self_name, &self.name)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct FnParamMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Duration,
    SystemTime,
    Enum {
        name: String,
    },
    Record {
        name: String,
    },
    ArcObject {
        object_name: String,
        is_trait: bool,
    },
    Error {
        name: String,
    },
    CallbackInterface {
        name: String,
    },
    Custom {
        name: String,
        builtin: Box<Type>,
    },
    Option {
        inner_type: Box<Type>,
    },
    Vec {
        inner_type: Box<Type>,
    },
    HashMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RecordMetadata {
    pub module_path: String,
    pub name: String,
    pub fields: Vec<FieldMetadata>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct FieldMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EnumMetadata {
    pub module_path: String,
    pub name: String,
    pub variants: Vec<VariantMetadata>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct VariantMetadata {
    pub name: String,
    pub fields: Vec<FieldMetadata>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ObjectMetadata {
    pub module_path: String,
    pub name: String,
    pub is_trait: bool,
}

impl ObjectMetadata {
    /// FFI symbol name for the `free` function for this object.
    ///
    /// This function is used to free the memory used by this object.
    pub fn free_ffi_symbol_name(&self) -> String {
        free_fn_symbol_name(&self.module_path, &self.name)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ErrorMetadata {
    pub module_path: String,
    pub name: String,
    pub variants: Vec<VariantMetadata>,
    pub flat: bool,
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
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Metadata {
    Namespace(NamespaceMetadata),
    Func(FnMetadata),
    Constructor(ConstructorMetadata),
    Method(MethodMetadata),
    Record(RecordMetadata),
    Enum(EnumMetadata),
    Object(ObjectMetadata),
    Error(ErrorMetadata),
}

impl Metadata {
    pub fn read(data: &[u8]) -> anyhow::Result<Self> {
        read_metadata(data)
    }
}

impl From<NamespaceMetadata> for Metadata {
    fn from(value: NamespaceMetadata) -> Metadata {
        Self::Namespace(value)
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

impl From<ErrorMetadata> for Metadata {
    fn from(v: ErrorMetadata) -> Self {
        Self::Error(v)
    }
}
