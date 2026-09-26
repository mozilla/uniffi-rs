/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

// Create 2 custom types on the Rust side.
//
// Bindings tests will typically customize only one of them on the foreign side.
pub struct CustomType1(u64);
pub struct CustomType2(u64);

uniffi::custom_type!(CustomType1, u64, {
    try_lift: |val| Ok(CustomType1(val)),
    lower: |custom| custom.0,
});
uniffi::custom_type!(CustomType2, u64, {
    try_lift: |val| Ok(CustomType2(val)),
    lower: |custom| custom.0,
});

/// Custom type that lowers another user-defined type
#[derive(uniffi::Object)]
pub struct CustomTypeInterface {
    value: u64,
}

#[uniffi::export]
impl CustomTypeInterface {
    #[uniffi::constructor]
    pub fn new(value: u64) -> Arc<Self> {
        Arc::new(Self { value })
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }
}

pub struct CustomType3(u64);

uniffi::custom_type!(CustomType3, Arc<CustomTypeInterface>, {
    try_lift: |int| Ok(CustomType3(int.value)),
    lower: |custom| CustomTypeInterface::new(custom.0),
});

#[uniffi::export]
pub fn roundtrip_custom_type1(custom1: CustomType1) -> CustomType1 {
    custom1
}

#[uniffi::export]
pub fn roundtrip_custom_type2(custom2: CustomType2) -> CustomType2 {
    custom2
}

#[uniffi::export]
pub fn roundtrip_custom_type3(custom3: CustomType3) -> CustomType3 {
    custom3
}
