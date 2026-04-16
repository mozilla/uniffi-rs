/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Extremely simple fixture to test making scaffolding calls without any arguments or return types

/// Test that bindings the bindings can make a call to `test_func`.  If in doubt, run the tests with
/// `--features=ffi-trace` to check that the function is actually called.
#[uniffi::export]
pub fn test_func() {}

/// Test what happens when there's an unexpected error in the Rust function, like a panic.
#[uniffi::export]
pub fn test_unexpected_error_func() {
    panic!("test panic")
}
