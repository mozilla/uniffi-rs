/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniffi::setup_scaffolding!("uniffi_bindgen_tests_external_types_source");

#[derive(uniffi::Record)]
pub struct ExternalRec {
    a: u8,
}

#[derive(uniffi::Enum)]
pub enum ExternalEnum {
    One,
    Two,
    Three,
}

#[derive(uniffi::Object)]
pub struct ExternalInterface {
    value: u32,
}

#[uniffi::export]
impl ExternalInterface {
    #[uniffi::constructor]
    fn new(value: u32) -> Self {
        Self { value }
    }

    fn get_value(&self) -> u32 {
        self.value
    }
}

pub struct ExternalCustomType(u64);

uniffi::custom_type!(ExternalCustomType, u64, {
    try_lift: |val| Ok(ExternalCustomType(val)),
    lower: |custom| custom.0,
});
