/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use uniffi_bindgen_tests_external_types_source::{
    ExternalCustomType, ExternalEnum, ExternalInterface, ExternalRec,
};

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
