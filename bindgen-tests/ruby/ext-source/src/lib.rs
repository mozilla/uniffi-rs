/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Defining crate for the Ruby external-type tests.
//!
//! Kept out of `bindgen-tests/external-types` so the shared suite stays what
//! Swift, Kotlin, Python, and the JNI adapter already compile.

use std::sync::Arc;

uniffi::setup_scaffolding!("uniffi_bindgen_tests_ruby_ext_source");

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct ExternalRec {
    pub a: u8,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
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
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }
}

pub struct ExternalUrl(String);

uniffi::custom_type!(ExternalUrl, String, {
    try_lift: |val| Ok(ExternalUrl(val)),
    lower: |custom| custom.0,
});

/// Nested fields of this record are also defined in this crate. Serializing it
/// from a consumer still has to call this crate's readers/writers for `en`/`rec`.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct ExternalNestedRec {
    pub en: ExternalEnum,
    pub rec: ExternalRec,
}

pub struct NestedExternalRec(pub ExternalRec);

uniffi::custom_newtype!(NestedExternalRec, ExternalRec);

pub struct NestedExternalInterface(pub Arc<ExternalInterface>);

uniffi::custom_newtype!(NestedExternalInterface, Arc<ExternalInterface>);

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum ExternalError {
    #[error("Boom")]
    Boom,
}
