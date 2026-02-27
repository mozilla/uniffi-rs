/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io;

/// Error enum
#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum TestError {
    #[error("Failure1")]
    Failure1,
    #[error("Failure2(data={data})")]
    Failure2 { data: String },
}

/// Flat error enum
///
/// The associated data for this won't be across the FFI.  Flat errors are mostly just a historical
/// artifact, but there are some use-cases for them -- for example variants that wrap errors from
/// other crates.
#[derive(uniffi::Error, thiserror::Error, Debug)]
#[uniffi(flat_error)]
pub enum TestFlatError {
    /// Here's an example of a variant that only works with a flat error, `io::Error` is not
    /// UniFFI compatible and can't be passed across the FFI.
    #[error("Failure1")]
    IoError(io::Error),
}

#[uniffi::export]
pub fn func_with_error(input: u32) -> Result<(), TestError> {
    match input {
        0 => Err(TestError::Failure1),
        1 => Err(TestError::Failure2 {
            data: "DATA".to_string(),
        }),
        _ => Ok(()),
    }
}

#[uniffi::export]
pub fn func_with_flat_error(input: u32) -> Result<(), TestFlatError> {
    match input {
        0 => Err(TestFlatError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "NotFound".to_string(),
        ))),
        _ => Ok(()),
    }
}
