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
pub struct NestedRecord {
    // This used to result in an error in bindings generation
    user_type_in_builtin_generic: Option<Two>,
}

#[derive(uniffi::Record)]
pub struct Three {
    obj: Arc<Object>,
}

#[derive(uniffi::Object)]
pub struct Object;

#[uniffi::export]
impl Object {
    fn is_heavy(&self) -> MaybeBool {
        MaybeBool::Uncertain
    }
}

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

#[derive(uniffi::Enum)]
pub enum MaybeBool {
    True,
    False,
    Uncertain,
}

#[uniffi::export]
fn enum_identity(value: MaybeBool) -> MaybeBool {
    value
}

include!(concat!(env!("OUT_DIR"), "/proc-macro.uniffi.rs"));

mod uniffi_types {
    pub use crate::{MaybeBool, NestedRecord, Object, One, Three, Two};
}
