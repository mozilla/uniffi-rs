/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::records::SimpleRec;

#[derive(uniffi::Enum)]
pub enum EnumNoData {
    A,
    B,
    C,
}

#[derive(uniffi::Enum)]
pub enum EnumWithData {
    A { value: u8 },
    B { value: String },
    C,
}

#[derive(uniffi::Enum)]
pub enum ComplexEnum {
    A { value: EnumNoData },
    B { value: EnumWithData },
    C { value: SimpleRec },
}

// Test enum with explicit discriminant values and gaps
#[repr(u8)]
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum ExplicitValuedEnum {
    First = 1,
    Second = 2,
    Fourth = 4,
    Tenth = 10,
    Eleventh = 11,
    Thirteenth = 13,
}

// Example with sequential and explicit values mixed
#[repr(u8)]
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum GappedEnum {
    One = 10,
    Two, // should be 11
    Three = 14,
}

#[uniffi::export]
pub fn roundtrip_enum_no_data(en: EnumNoData) -> EnumNoData {
    en
}

#[uniffi::export]
pub fn roundtrip_enum_with_data(en: EnumWithData) -> EnumWithData {
    en
}

#[uniffi::export]
pub fn roundtrip_complex_enum(en: ComplexEnum) -> ComplexEnum {
    en
}
