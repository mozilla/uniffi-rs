/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Error, Debug)]
pub enum SimpleError {
    InvalidInput,
    OsError,
}

#[derive(uniffi::Enum)]
pub enum SimpleEnum {
    One,
    Two,
}

#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum ReprU8 {
    One = 1,
    Two = 0x2,
}

#[derive(uniffi::Enum)]
pub enum ComplexEnum {
    None,
    String(String),
    Int(i64),
    All { s: String, i: i64 },
    Vec(Vec<String>),
}

#[derive(uniffi::Record)]
pub struct SimpleRecord {
    string: String,
    boolean: bool,
    integer: i32,
    float_var: f64,
    vec: Vec<bool>,
}

#[derive(uniffi::Record)]
pub struct RecordWithOptionals {
    string: Option<String>,
    boolean: Option<bool>,
    integer: Option<i32>,
    float_var: Option<f64>,
    vec: Option<Vec<String>>,
}

#[derive(uniffi::Record)]
pub struct MultiLayerRecord {
    simple_enum: SimpleEnum,
    repr_u8: ReprU8,
    simple_record: SimpleRecord,
}

uniffi::include_scaffolding!("codable_test");
