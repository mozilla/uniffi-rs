/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

#[derive(uniffi::Object)]
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

    // Test that uniffi::export can handle Self in arbitrary positions
    fn some_method(self: Arc<Self>) -> Option<Arc<Self>> {
        None
    }
}

uniffi::include_namespaced_scaffolding!("uniffi_simple_iface");
