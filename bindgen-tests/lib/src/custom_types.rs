/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

#[uniffi::export]
pub fn roundtrip_custom_type1(custom1: CustomType1) -> CustomType1 {
    custom1
}

#[uniffi::export]
pub fn roundtrip_custom_type2(custom2: CustomType2) -> CustomType2 {
    custom2
}
