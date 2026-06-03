/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Tests for inputting and returning `Option<T>` arguments

// Rec that's contained inside options
#[derive(uniffi::Record)]
pub struct OptionsRec {
    pub a: u8,
}

#[uniffi::export]
pub fn roundtrip_option_u8(a: Option<u8>) -> Option<u8> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_i8(a: Option<i8>) -> Option<i8> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_u16(a: Option<u16>) -> Option<u16> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_i16(a: Option<i16>) -> Option<i16> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_u32(a: Option<u32>) -> Option<u32> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_i32(a: Option<i32>) -> Option<i32> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_u64(a: Option<u64>) -> Option<u64> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_i64(a: Option<i64>) -> Option<i64> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_f32(a: Option<f32>) -> Option<f32> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_f64(a: Option<f64>) -> Option<f64> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_string(a: Option<String>) -> Option<String> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_bool(a: Option<bool>) -> Option<bool> {
    a
}

#[uniffi::export]
pub fn roundtrip_option_rec(a: Option<OptionsRec>) -> Option<OptionsRec> {
    a
}
