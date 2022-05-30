/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
fn byte_to_u32(byte: u8) -> u32 {
    byte.into()
}

include!(concat!(env!("OUT_DIR"), "/simple-fns.uniffi.rs"));
