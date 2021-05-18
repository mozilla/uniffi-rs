/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub fn returns_u16() -> u16 {
    16
}

pub fn accepts_and_returns_unsigned(a: u8, b: u16, c: u32, d: u64) -> u64 {
    (a as u64) + (b as u64) + (c as u64) + d
}

pub struct DirectlyUsesU8 {
    member: u8,
    other: String,
}

pub struct RecursivelyUsesU8 {
    member: DirectlyUsesU8,
}

include!(concat!(env!("OUT_DIR"), "/test.uniffi.rs"));
