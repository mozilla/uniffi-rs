/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Record, Clone)]
pub struct SimpleRec {
    pub a: u8,
}

#[uniffi::export]
impl SimpleRec {
    pub fn roundtrip(&self) -> Self {
        SimpleRec { a: self.a }
    }
}

#[derive(uniffi::Record)]
pub struct UnitRec;

// Record where the first field has no data
#[derive(uniffi::Record)]
pub struct RecWithUnitField {
    pub unit: UnitRec,
    pub a: u8,
}

// Note: tuple-style records are currently not supported

#[derive(uniffi::Record, Clone)]
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
    pub field_string: String,
    pub field_rec: SimpleRec,
}

// Record with a large number of fields, counting the ones in nested records.
// Bindings that pass each field as a separate FFI argument may need to special-case this.
#[derive(uniffi::Record, Clone)]
pub struct LargeRec {
    pub a: ComplexRec,
    pub b: ComplexRec,
    pub c: ComplexRec,
}

#[uniffi::export]
impl LargeRec {
    pub fn roundtrip(&self) -> Self {
        self.clone()
    }
}

// Record with more than 255 fields, counting the ones in nested records.
// 255 is the maximum number of arguments for a JVM method.
#[derive(uniffi::Record, Clone)]
pub struct NestedLargeRec {
    pub a: LargeRec,
    pub b: LargeRec,
    pub c: LargeRec,
    pub d: LargeRec,
    pub e: LargeRec,
    pub f: LargeRec,
}

#[uniffi::export]
pub fn roundtrip_simple_rec(rec: SimpleRec) -> SimpleRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_complex_rec(rec: ComplexRec) -> ComplexRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_rec_with_unit_field(rec: RecWithUnitField) -> RecWithUnitField {
    rec
}

#[uniffi::export]
pub fn roundtrip_large_rec(rec: LargeRec) -> LargeRec {
    rec
}

// Create a `LargeRec`, this saves the tests from spelling out all the fields
#[uniffi::export]
pub fn create_large_rec() -> LargeRec {
    let complex_rec = |n: u8| ComplexRec {
        field_u8: n,
        field_i8: -(n as i8),
        field_u16: n as u16 + 1,
        field_i16: -(n as i16) - 2,
        field_u32: n as u32 + 3,
        field_i32: -(n as i32) - 4,
        field_u64: n as u64 + 5,
        field_i64: -(n as i64) - 6,
        field_f32: n as f32 + 0.5,
        field_f64: n as f64 + 1.5,
        field_string: format!("test{n}"),
        field_rec: SimpleRec { a: n },
    };
    LargeRec {
        a: complex_rec(10),
        b: complex_rec(20),
        c: complex_rec(30),
    }
}

#[uniffi::export]
pub fn roundtrip_nested_large_rec(rec: NestedLargeRec) -> NestedLargeRec {
    rec
}

#[uniffi::export]
pub fn create_nested_large_rec() -> NestedLargeRec {
    NestedLargeRec {
        a: create_large_rec(),
        b: create_large_rec(),
        c: create_large_rec(),
        d: create_large_rec(),
        e: create_large_rec(),
        f: create_large_rec(),
    }
}
