/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Intermediate crate: its public type embeds types from
//! `uniffi-bindgen-tests-external-types-source`. A consumer that only names
//! `MidRec` does not list those nested types in its own interface.

uniffi::setup_scaffolding!("uniffi_bindgen_tests_mid_types");

use uniffi_bindgen_tests_external_types_source::{ExternalEnum, ExternalRec};

#[derive(uniffi::Record)]
pub struct MidRec {
    pub inner: ExternalRec,
    pub maybe_enum: Option<ExternalEnum>,
}
