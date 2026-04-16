/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Record)]
pub struct SimpleRec {
    pub a: u8,
}

#[derive(uniffi::Record)]
pub struct RecWithDefault {
    #[uniffi(default = 42)]
    pub a: u8,
}

#[derive(uniffi::Record)]
pub struct ComplexRec {
    pub field_u8: u8,
    pub field_i8: i8,
    pub field_u16: u16,
    pub field_i16: i16,
    pub field_u32: u32,
    pub field_i32: i32,
    pub field_u64: u64,
    pub field_i64: i64,
    pub field_f32: f32,
    pub field_f64: f64,
    #[uniffi(default = "DefaultString")]
    pub field_string: String,
    pub field_rec: SimpleRec,
}

#[uniffi::export]
pub fn roundtrip_simple_rec(rec: SimpleRec) -> SimpleRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_complex_rec(rec: ComplexRec) -> ComplexRec {
    rec
}
