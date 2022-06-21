/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

pub fn empty_func() {
    // Intentionally left empty
}

pub fn get_dict() -> HashMap<String, String> {
    HashMap::default()
}

include!(concat!(env!("OUT_DIR"), "/test.uniffi.rs"));
