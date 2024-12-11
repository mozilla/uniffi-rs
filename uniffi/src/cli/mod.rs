/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod swift;
mod uniffi_bindgen;

pub fn uniffi_bindgen_main() {
    if let Err(e) = uniffi_bindgen::run_main() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}

pub fn uniffi_bindgen_swift() {
    if let Err(e) = swift::run_main() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
