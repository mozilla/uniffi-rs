/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Backend-agnostic askama filters

use crate::interface::{AsType, FfiType};
use std::fmt;

// convert `anyhow::Error` and `&str` etc to askama errors.
// should only be needed by "filters", otherwise anyhow etc work directly.
pub fn to_askama_error<T: ToString + ?Sized>(t: &T) -> askama::Error {
    askama::Error::Custom(Box::new(FilterError(t.to_string())))
}

// Need a struct to define an error that implements std::error::Error, which neither String nor
// anyhow::Error do.
#[derive(Debug)]
struct FilterError(String);

impl fmt::Display for FilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FilterError {}

/// Get the FfiType for a Type
pub fn ffi_type(type_: &impl AsType) -> askama::Result<FfiType, askama::Error> {
    Ok(type_.as_type().into())
}
