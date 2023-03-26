/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    // Force a rebuild when we set this env var to force a contract version mismatch
    println!("cargo:rerun-if-env-changed=UNIFFI_FORCE_CONTRACT_VERSION");
    uniffi::generate_scaffolding("./src/api_v1.udl").unwrap();
}
