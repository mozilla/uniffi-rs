/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

mod kotlin;
pub use kotlin::{
    run_script as kotlin_run_script, run_test as kotlin_run_test, KotlinBindingGenerator,
};
mod python;
pub use python::{
    run_script as python_run_script, run_test as python_run_test, PythonBindingGenerator,
};
mod ruby;
pub use ruby::{run_test as ruby_run_test, RubyBindingGenerator};
mod swift;
pub use swift::{
    run_script as swift_run_script, run_test as swift_run_test, SwiftBindingGenerator,
};

/// Mode for the `run_script` function defined for each language
#[derive(Clone, Debug)]
pub struct RunScriptOptions {
    pub show_compiler_messages: bool,
}

impl Default for RunScriptOptions {
    fn default() -> Self {
        Self {
            show_compiler_messages: true,
        }
    }
}
