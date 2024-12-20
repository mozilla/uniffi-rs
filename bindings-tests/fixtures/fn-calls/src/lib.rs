//! Extremely simple fixture to test making scaffolding calls without any arguments or return types
//!
//! The test is if the bindings can make a call to `test_func`.  If in doubt, run the tests with
//! `--features=ffi-trace` to check that the function is actually called.

#[uniffi::export]
pub fn test_func() {}

uniffi::setup_scaffolding!("fn_calls");
