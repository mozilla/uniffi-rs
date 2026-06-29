/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod method_names;

#[test]
fn test_generated_code_compiles() {
    // No code here, since we're just testing if the generated code can be compiled.
}

uniffi::setup_scaffolding!();
