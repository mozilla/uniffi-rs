/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

trait ForeignGetters {
    fn get_bool(&self, v: bool, argument_two: bool) -> Result<bool, SimpleError>;
    fn get_string(&self, v: String, arg2: bool) -> Result<String, SimpleError>;
    fn get_option(&self, v: Option<String>, arg2: bool) -> Result<Option<String>, ComplexError>;
    fn get_list(&self, v: Vec<i32>, arg2: bool) -> Result<Vec<i32>, SimpleError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SimpleError {
    #[error("BadArgument")]
    BadArgument,
    #[error("InternalTelephoneError")]
    UnexpectedError,
}

#[derive(Debug, thiserror::Error)]
pub enum ComplexError {
    #[error("ReallyBadArgument")]
    ReallyBadArgument { code: i32 },
    #[error("InternalTelephoneError")]
    UnexpectedErrorWithReason { reason: String },
}

impl From<uniffi::UnexpectedUniFFICallbackError> for SimpleError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> SimpleError {
        SimpleError::UnexpectedError
    }
}

impl From<uniffi::UnexpectedUniFFICallbackError> for ComplexError {
    fn from(e: uniffi::UnexpectedUniFFICallbackError) -> ComplexError {
        ComplexError::UnexpectedErrorWithReason { reason: e.reason }
    }
}

#[derive(Debug, Clone)]
pub struct RustGetters;

impl RustGetters {
    pub fn new() -> Self {
        RustGetters
    }
    fn get_bool(
        &self,
        callback: Box<dyn ForeignGetters>,
        v: bool,
        argument_two: bool,
    ) -> Result<bool, SimpleError> {
        callback.get_bool(v, argument_two)
    }
    fn get_string(
        &self,
        callback: Box<dyn ForeignGetters>,
        v: String,
        arg2: bool,
    ) -> Result<String, SimpleError> {
        callback.get_string(v, arg2)
    }
    fn get_option(
        &self,
        callback: Box<dyn ForeignGetters>,
        v: Option<String>,
        arg2: bool,
    ) -> Result<Option<String>, ComplexError> {
        callback.get_option(v, arg2)
    }
    fn get_list(
        &self,
        callback: Box<dyn ForeignGetters>,
        v: Vec<i32>,
        arg2: bool,
    ) -> Result<Vec<i32>, SimpleError> {
        callback.get_list(v, arg2)
    }

    fn get_string_optional_callback(
        &self,
        callback: Option<Box<dyn ForeignGetters>>,
        v: String,
        arg2: bool,
    ) -> Result<Option<String>, SimpleError> {
        callback.map(|c| c.get_string(v, arg2)).transpose()
    }
}

impl Default for RustGetters {
    fn default() -> Self {
        Self::new()
    }
}

// Use `Send+Send` because we want to store the callback in an exposed
// `Send+Sync` object.
#[allow(clippy::wrong_self_convention)]
trait StoredForeignStringifier: Send + Sync + std::fmt::Debug {
    fn from_simple_type(&self, value: i32) -> String;
    fn from_complex_type(&self, values: Option<Vec<Option<f64>>>) -> String;
}

#[derive(Debug)]
pub struct RustStringifier {
    callback: Box<dyn StoredForeignStringifier>,
}

impl RustStringifier {
    fn new(callback: Box<dyn StoredForeignStringifier>) -> Self {
        RustStringifier { callback }
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_simple_type(&self, value: i32) -> String {
        self.callback.from_simple_type(value)
    }
}

include!(concat!(env!("OUT_DIR"), "/callbacks.uniffi.rs"));
