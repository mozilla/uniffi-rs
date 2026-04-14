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

    #[uniffi::constructor]
    pub fn secondary_constructor(value: u32) -> Self {
        Self { value: value * 2 }
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

    /// Test a multi-word argument.  `the_argument` should be normalized to the naming style of the
    /// foreign language.
    pub fn method_with_multi_word_arg(&self, the_argument: String) -> String {
        the_argument
    }
}

#[uniffi::export]
pub fn clone_interface(interface: Arc<TestInterface>) -> Arc<TestInterface> {
    interface
}

// Test interfaces in records
#[derive(uniffi::Record)]
pub struct TwoTestInterfaces {
    pub first: Arc<TestInterface>,
    pub second: Arc<TestInterface>,
}

#[uniffi::export]
pub fn swap_test_interfaces(interfaces: TwoTestInterfaces) -> TwoTestInterfaces {
    TwoTestInterfaces {
        first: interfaces.second,
        second: interfaces.first,
    }
}

// Test interfaces in enums
#[derive(uniffi::Enum)]
pub enum TestInterfaceEnum {
    One { i: Arc<TestInterface> },
    Two { i: Arc<TestInterface> },
}
