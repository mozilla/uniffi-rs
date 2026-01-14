/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Test lifting/lowering primitive types

#[uniffi::export(default(arg = "DEFAULT"))]
pub fn func_with_default(arg: String) -> String {
    arg
}

/// Test a multi-word argument.  `the_argument` should be normalized to the naming style of the
/// foreign language.
#[uniffi::export]
pub fn func_with_multi_word_arg(the_argument: String) -> String {
    the_argument
}

#[derive(uniffi::Object)]
struct ComplexMethods;

#[uniffi::export]
impl ComplexMethods {
    #[uniffi::constructor()]
    pub fn new() -> Self {
        Self
    }

    #[uniffi::method(default(arg = "DEFAULT"))]
    pub fn method_with_default(&self, arg: String) -> String {
        arg
    }

    pub fn method_with_multi_word_arg(&self, the_argument: String) -> String {
        the_argument
    }
}
