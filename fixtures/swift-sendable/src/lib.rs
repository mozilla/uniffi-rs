#![allow(dead_code)]

use std::sync::Arc;

#[derive(Debug, uniffi::Record)]
struct UniffiRecord {
    value: String,
}

#[derive(Debug, uniffi::Object)]
struct UniffiObject {
    value: String,
}

#[derive(Debug, uniffi::Record)]
struct Nested {
    value: Arc<RecordAtLayer1>,
}

#[derive(Debug, uniffi::Object)]
struct RecordAtLayer1 {
    value: RecordAtLayer2,
}

#[derive(Debug, uniffi::Record)]
struct RecordAtLayer2 {}

uniffi::setup_scaffolding!("swift_sendable");
