/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use uniffi_bindgen_tests_external_types_source::{
    ExternalCustomType, ExternalEnum, ExternalError, ExternalInterface, ExternalNestedRec,
    ExternalRec, ExternalUrl, NestedExternalInterface, NestedExternalRec,
};
use uniffi_bindgen_tests_mid_types::MidRec;

#[uniffi::export]
pub fn roundtrip_ext_record(rec: ExternalRec) -> ExternalRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_ext_enum(en: ExternalEnum) -> ExternalEnum {
    en
}

#[uniffi::export]
pub fn roundtrip_ext_interface(interface: Arc<ExternalInterface>) -> Arc<ExternalInterface> {
    interface
}

#[uniffi::export]
pub fn roundtrip_ext_custom_type(custom: ExternalCustomType) -> ExternalCustomType {
    custom
}

#[uniffi::export]
pub fn roundtrip_ext_url(url: ExternalUrl) -> ExternalUrl {
    url
}

#[uniffi::export]
pub fn roundtrip_ext_nested_rec(rec: ExternalNestedRec) -> ExternalNestedRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_nested_ext_rec(rec: NestedExternalRec) -> NestedExternalRec {
    rec
}

#[uniffi::export]
pub fn roundtrip_nested_ext_interface(
    interface: NestedExternalInterface,
) -> NestedExternalInterface {
    interface
}

#[uniffi::export]
pub fn roundtrip_maybe_ext_enum(en: Option<ExternalEnum>) -> Option<ExternalEnum> {
    en
}

#[uniffi::export]
pub fn roundtrip_ext_enums(ens: Vec<ExternalEnum>) -> Vec<ExternalEnum> {
    ens
}

#[uniffi::export]
pub async fn async_roundtrip_ext_enum(en: ExternalEnum) -> ExternalEnum {
    en
}

#[uniffi::export]
pub fn throw_ext_error() -> Result<(), ExternalError> {
    Err(ExternalError::Boom)
}

/// Local custom type whose builtin is an imported custom type with a bindings conversion.
pub struct LocalUrl(pub ExternalUrl);

uniffi::custom_newtype!(LocalUrl, ExternalUrl);

#[uniffi::export]
pub fn roundtrip_local_url(url: LocalUrl) -> LocalUrl {
    url
}

#[uniffi::export]
pub fn roundtrip_mid_rec(rec: MidRec) -> MidRec {
    rec
}
