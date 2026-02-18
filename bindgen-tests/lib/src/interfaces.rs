/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct TestInterface {
    value: u32,
}

#[uniffi::export]
impl TestInterface {
    #[uniffi::constructor]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }

    /// Get the current reference count for this object
    ///
    /// The count does not include the extra reference needed to call this method.
    pub fn ref_count(self: Arc<Self>) -> u32 {
        (Arc::strong_count(&self) - 1) as u32
    }
}

#[uniffi::export]
pub fn clone_interface(interface: Arc<TestInterface>) -> Arc<TestInterface> {
    interface
}

// Test interfaces in records
#[derive(uniffi::Record)]
pub struct TwoTestInterfaces {
    first: Arc<TestInterface>,
    second: Arc<TestInterface>,
}

#[uniffi::export]
pub fn swap_test_interfaces(interfaces: TwoTestInterfaces) -> TwoTestInterfaces {
    TwoTestInterfaces {
        first: interfaces.second,
        second: interfaces.first,
    }
}
