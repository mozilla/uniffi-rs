/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, thiserror::Error)]
pub enum TelephoneError {
    #[error("Busy")]
    Busy,
    #[error("InternalTelephoneError")]
    InternalTelephoneError,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for TelephoneError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::InternalTelephoneError
    }
}

pub trait CallAnswerer {
    fn answer(&self) -> Result<String, TelephoneError>;
}

#[derive(Debug, Clone)]
pub struct Telephone;
impl Telephone {
    pub fn new() -> Self {
        Telephone
    }

    pub fn call(&self, answerer: Box<dyn CallAnswerer>) -> Result<String, TelephoneError> {
        answerer.answer()
    }
}

include!(concat!(env!("OUT_DIR"), "/callbacks.uniffi.rs"));
