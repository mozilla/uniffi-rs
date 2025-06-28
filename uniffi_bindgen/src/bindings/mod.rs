/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

mod kotlin;
pub use kotlin::KotlinBindingGenerator;
pub mod python;
mod ruby;
pub use ruby::RubyBindingGenerator;
mod swift;
pub use swift::{generate_swift_bindings, SwiftBindingGenerator, SwiftBindingsOptions};

#[cfg(feature = "bindgen-tests")]
pub use self::{
    kotlin::test as kotlin_test, python::test as python_test, ruby::test as ruby_test,
    swift::test as swift_test,
};

#[cfg(feature = "bindgen-tests")]
/// Mode for the `run_script` function defined for each language
#[derive(Clone, Debug)]
pub struct RunScriptOptions {
    pub show_compiler_messages: bool,
}

#[cfg(feature = "bindgen-tests")]
impl Default for RunScriptOptions {
    fn default() -> Self {
        Self {
            show_compiler_messages: true,
        }
    }
}
