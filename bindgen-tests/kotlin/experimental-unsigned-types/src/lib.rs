/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub fn returns_u16() -> u16 {
    16
}

pub fn accepts_and_returns_unsigned(a: u8, b: u16, c: u32, d: u64) -> u64 {
    (a as u64) + (b as u64) + (c as u64) + d
}

pub fn accepts_unsigned_returns_void(a: u8, b: u16, c: u32, d: u64) {
    let _unused = accepts_and_returns_unsigned(a, b, c, d);
}

pub struct DirectlyUsesU8 {
    member: u8,
    member_two: u16,
    member_three: u32,
    member_four: u64,
    other: String,
}

pub enum UnsignedEnum {
    V1 { q1: u8 },
    V2 { addr: String },
}

pub struct RecursivelyUsesU8 {
    member: DirectlyUsesU8,
    member_two: UnsignedEnum,
}

#[derive(Debug)]
pub struct InterfaceUsingUnsigned {
    _member: u64,
}

impl InterfaceUsingUnsigned {
    pub fn new(new: u64) -> Self {
        Self { _member: new }
    }

    fn uses_unsigned_struct(&self, mut p1: RecursivelyUsesU8) {
        p1.member_two = UnsignedEnum::V1 { q1: 0 }
    }
}

uniffi::include_scaffolding!("test");
