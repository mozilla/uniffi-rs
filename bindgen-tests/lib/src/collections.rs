/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

// Simple tests for inputting and returning arguments

// Rec that's contained inside vecs/maps
#[derive(uniffi::Record)]
pub struct CollectionsRec {
    pub a: u8,
}

// Rec that contains options/vecs/maps
#[derive(uniffi::Record)]
pub struct RecWithCollections {
    // Put `EnumWithCollections` first to exorcise the buffer packing code
    // It can be tricky to add the right padding for an enum that contains a dynamically sized type.
    pub a: EnumWithCollections,
    pub b: Option<u32>,
    pub c: Vec<bool>,
    pub d: HashMap<String, u8>,
}

#[derive(uniffi::Enum)]
pub enum EnumWithCollections {
    A(Option<u32>),
    B(Vec<bool>),
    C(HashMap<String, u8>),
}

// Note: no Vec<u8> test, since that's covered by the bytes test
#[uniffi::export]
pub fn roundtrip_vec_i8(a: Vec<i8>) -> Vec<i8> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_u16(a: Vec<u16>) -> Vec<u16> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_i16(a: Vec<i16>) -> Vec<i16> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_u32(a: Vec<u32>) -> Vec<u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_i32(a: Vec<i32>) -> Vec<i32> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_u64(a: Vec<u64>) -> Vec<u64> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_i64(a: Vec<i64>) -> Vec<i64> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_f32(a: Vec<f32>) -> Vec<f32> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_f64(a: Vec<f64>) -> Vec<f64> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_string(a: Vec<String>) -> Vec<String> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_bool(a: Vec<bool>) -> Vec<bool> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_rec(a: Vec<CollectionsRec>) -> Vec<CollectionsRec> {
    a
}

#[uniffi::export]
pub fn roundtrip_hash_map(a: HashMap<String, u32>) -> HashMap<String, u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_hash_set(a: HashSet<String>) -> HashSet<String> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec_hash_set(a: Option<Vec<HashSet<String>>>) -> Option<Vec<HashSet<String>>> {
    a
}

#[uniffi::export]
pub fn roundtrip_hash_map_u32_key(a: HashMap<u32, u32>) -> HashMap<u32, u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_rec_with_collections(a: RecWithCollections) -> RecWithCollections {
    a
}

#[derive(uniffi::Record)]
pub struct CollectionsComplexRec {
    pub a: u32,
    pub b: String,
    pub c: CollectionsEnum,
}

#[derive(uniffi::Enum)]
pub enum CollectionsEnum {
    A(i64),
    B { a: f32, b: bool },
}

#[uniffi::export]
pub fn roundtrip_complex_collection_type(
    a: Option<Vec<HashMap<String, CollectionsComplexRec>>>,
) -> Option<Vec<HashMap<String, CollectionsComplexRec>>> {
    a
}
