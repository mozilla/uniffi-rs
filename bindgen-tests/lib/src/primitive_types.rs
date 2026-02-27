/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Test lifting/lowering primitive types

// Simple tests for inputting and returning arguments

#[uniffi::export]
pub fn roundtrip_u8(a: u8) -> u8 {
    a
}

#[uniffi::export]
pub fn roundtrip_i8(a: i8) -> i8 {
    a
}

#[uniffi::export]
pub fn roundtrip_u16(a: u16) -> u16 {
    a
}

#[uniffi::export]
pub fn roundtrip_i16(a: i16) -> i16 {
    a
}

#[uniffi::export]
pub fn roundtrip_u32(a: u32) -> u32 {
    a
}

#[uniffi::export]
pub fn roundtrip_i32(a: i32) -> i32 {
    a
}

#[uniffi::export]
pub fn roundtrip_u64(a: u64) -> u64 {
    a
}

#[uniffi::export]
pub fn roundtrip_i64(a: i64) -> i64 {
    a
}

#[uniffi::export]
pub fn roundtrip_f32(a: f32) -> f32 {
    a
}

#[uniffi::export]
pub fn roundtrip_f64(a: f64) -> f64 {
    a
}

#[uniffi::export]
pub fn roundtrip_bool(a: bool) -> bool {
    a
}

#[uniffi::export]
pub fn roundtrip_string(a: String) -> String {
    a
}

/// Complex test: input a bunch of different values and add them together
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn sum_with_many_types(
    a: u8,
    b: i8,
    c: u16,
    d: i16,
    e: u32,
    f: i32,
    g: u64,
    h: i64,
    i: f32,
    j: f64,
    negate: bool,
) -> f64 {
    let all_values = [
        a as f64, b as f64, c as f64, d as f64, e as f64, f as f64, g as f64, h as f64, i as f64, j,
    ];
    let sum: f64 = all_values.into_iter().sum();
    if negate {
        -sum
    } else {
        sum
    }
}
