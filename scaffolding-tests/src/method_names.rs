/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This module tests method name conflicts
// https://github.com/mozilla/uniffi-rs/issues/2881

#[uniffi::export(rust, foreign)]
pub trait ExportedTrait: Send + Sync {
    fn conflicted_method_name(&self) -> String;
}

impl dyn ExportedTrait {
    // Inherit method with the same name as the trait method name.
    //
    // UniFFI should use a fully-qualified function syntax to ensure the other method is called.
    #[allow(unused)]
    fn conflicted_method_name(&self) {}
}
