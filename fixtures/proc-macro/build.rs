/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    // generate_scaffolding would work here, but we use the _for_crate version for
    // test coverage.
    uniffi::generate_scaffolding_for_crate("src/proc-macro.udl", "uniffi_proc_macro").unwrap();
}
