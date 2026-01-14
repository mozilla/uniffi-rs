/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

// Simple tests for inputting and returning arguments

#[uniffi::export]
pub fn roundtrip_option(a: Option<u32>) -> Option<u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_vec(a: Vec<u32>) -> Vec<u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_hash_map(a: HashMap<String, u32>) -> HashMap<String, u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_complex_compound(
    a: Option<Vec<HashMap<String, u32>>>,
) -> Option<Vec<HashMap<String, u32>>> {
    a
}
