/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    pub return_type: Option<Type>,
}

impl FnMetadata {
    pub fn ffi_symbol_name(&self) -> String {
        fn_ffi_symbol_name(&self.module_path, &self.name, checksum(self))
    }
}

#[derive(Debug, Hash, Deserialize, Serialize)]
pub struct FnParamMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Debug, Hash, Deserialize, Serialize)]
#[serde(tag = "id")]
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

pub fn fn_ffi_symbol_name(mod_path: &[String], name: &str, checksum: u16) -> String {
    let mod_path = mod_path.join("__");
    format!("_uniffi_{mod_path}_{name}_{checksum:x}")
}
