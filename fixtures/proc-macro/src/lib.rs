/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}

#[derive(uniffi::Record)]
pub struct Two {
    a: String,
    b: Option<Vec<bool>>,
}

#[derive(uniffi::Record)]
pub struct Three {
    obj: Arc<Object>,
}

#[derive(uniffi::Object)]
pub struct Object;

#[uniffi::export]
fn make_one(inner: i32) -> One {
    One { inner }
}

#[uniffi::export]
fn take_two(two: Two) -> String {
    two.a
}

#[uniffi::export]
fn make_object() -> Arc<Object> {
    Arc::new(Object)
}

include!(concat!(env!("OUT_DIR"), "/proc-macro.uniffi.rs"));

mod uniffi_types {
    pub use crate::{Object, One, Three, Two};
}
