use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, Deserialize, Serialize)]
pub struct FnMetadata {
    pub module_path: Vec<String>,
    pub name: String,
    pub inputs: Vec<FnParamMetadata>,
    pub output: Option<Type>,
}

#[derive(Debug, Hash, Deserialize, Serialize)]
pub struct FnParamMetadata {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Hash, Deserialize, Serialize)]
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
}

/// Returns the last 16 bits of the value's hash as computed with [`DefaultHasher`].
///
/// To be used as a checksum of FFI symbols, as a safeguard against different UniFFI versions being
/// used for scaffolding and bindings generation.
pub fn checksum<T: Hash>(val: &T) -> u16 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    (hasher.finish() & 0x000000000000FFFF) as u16
}
