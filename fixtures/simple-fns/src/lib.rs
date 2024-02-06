/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub type MyHashSet = Mutex<HashSet<String>>;

#[uniffi::export]
fn get_string() -> String {
    "String created by Rust".to_owned()
}

#[uniffi::export]
fn get_int() -> i32 {
    1289
}

#[uniffi::export]
fn string_identity(s: String) -> String {
    s
}

#[uniffi::export]
fn hash_map_identity(h: HashMap<String, String>) -> HashMap<String, String> {
    h
}

#[uniffi::export]
fn byte_to_u32(byte: u8) -> u32 {
    byte.into()
}

#[uniffi::export]
fn new_set() -> Arc<MyHashSet> {
    Arc::default()
}

#[uniffi::export]
fn add_to_set(set: Arc<MyHashSet>, value: String) {
    set.lock().unwrap().insert(value);
}

#[uniffi::export]
fn set_contains(set: Arc<MyHashSet>, value: String) -> bool {
    set.lock().unwrap().contains(&value)
}

// This used to generate broken bindings because the type inside `Option` (and
// other generic builtin types) wasn't being added as a known type.
#[uniffi::export]
fn dummy(_arg: Option<i8>) {}

uniffi::include_scaffolding!("simple-fns");
