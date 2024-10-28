/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generate foreign language bindings for a uniffi component.
//!
//! This module contains all the code for generating foreign language bindings,
//! along with some helpers for executing foreign language scripts or tests.

#[cfg(feature = "kotlin")]
mod kotlin;
#[cfg(feature = "kotlin")]
pub use kotlin::KotlinBindingGenerator;

#[cfg(feature = "python")]
mod python;
#[cfg(feature = "python")]
pub use python::PythonBindingGenerator;

#[cfg(feature = "ruby")]
mod ruby;
#[cfg(feature = "ruby")]
pub use ruby::RubyBindingGenerator;

#[cfg(feature = "swift")]
mod swift;
#[cfg(feature = "swift")]
pub use swift::{generate_swift_bindings, SwiftBindingGenerator, SwiftBindingsOptions};

mod backend;

#[cfg(all(feature = "bindgen-tests", feature = "kotlin"))]
pub use self::kotlin::test as kotlin_test;

#[cfg(all(feature = "bindgen-tests", feature = "python"))]
pub use self::python::test as python_test;

#[cfg(all(feature = "bindgen-tests", feature = "ruby"))]
pub use self::ruby::test as ruby_test;

#[cfg(all(feature = "bindgen-tests", feature = "swift"))]
pub use self::swift::test as swift_test;

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
