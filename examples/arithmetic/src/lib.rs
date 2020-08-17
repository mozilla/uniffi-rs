/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, thiserror::Error)]
enum ArithmeticError {
    #[error("Integer overflow on an operation with {a} and {b}")]
    IntegerOverflow { a: u64, b: u64 },
}

fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or(ArithmeticError::IntegerOverflow { a, b })
}

fn sub(a: u64, b: u64) -> Result<u64> {
    a.checked_sub(b)
        .ok_or(ArithmeticError::IntegerOverflow { a, b })
}

fn equal(a: u64, b: u64) -> bool {
    a == b
}

type Result<T, E = ArithmeticError> = std::result::Result<T, E>;

include!(concat!(env!("OUT_DIR"), "/arithmetic.uniffi.rs"));
