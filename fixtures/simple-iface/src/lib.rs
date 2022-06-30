/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

pub struct Object {
    inner: i32,
}

#[uniffi::export]
fn make_object(inner: i32) -> Arc<Object> {
    Arc::new(Object { inner })
}

#[uniffi::export]
impl Object {
    fn get_inner(&self) -> i32 {
        self.inner
    }
}

include!(concat!(env!("OUT_DIR"), "/simple-iface.uniffi.rs"));

mod uniffi_types {
    pub use crate::Object;
}
