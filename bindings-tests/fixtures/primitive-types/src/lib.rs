//! Test lifting/lowering primitive types

// Simple tests

#[uniffi::export]
pub fn roundtrip(a: u32) -> u32 {
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
pub fn sum(
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

uniffi::setup_scaffolding!("primitive_types");
