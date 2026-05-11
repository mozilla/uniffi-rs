/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

// Function that inputs an argument by reference
#[uniffi::export]
pub fn roundtrip_u8_ref(a: &u8) -> u8 {
    *a
}

// Input an interface reference
#[uniffi::export]
pub fn call_double_value(i: &ReferenceTestInterface, a: u32) -> u32 {
    i.double_value(&a)
}

// Input an trait interface reference
#[uniffi::export]
pub fn call_triple_value_trait_interface(t: &dyn ReferenceTestTraitInterface, a: u32) -> u32 {
    t.triple_value(a)
}

#[derive(uniffi::Object, Default)]
pub struct ReferenceTestInterface;

#[uniffi::export]
impl ReferenceTestInterface {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    // Test a method that inputs a reference
    pub fn double_value(&self, a: &u32) -> u32 {
        a * 2
    }
}

#[uniffi::export]
pub trait ReferenceTestTraitInterface: Send + Sync {
    fn triple_value(&self, a: u32) -> u32;
}

#[uniffi::export]
pub fn create_reference_test_trait_interface() -> Arc<dyn ReferenceTestTraitInterface> {
    Arc::new(ReferenceTestTraitInterfaceImpl)
}

struct ReferenceTestTraitInterfaceImpl;

impl ReferenceTestTraitInterface for ReferenceTestTraitInterfaceImpl {
    fn triple_value(&self, a: u32) -> u32 {
        a * 3
    }
}
