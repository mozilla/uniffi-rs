use std::convert::TryInto;

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(Debug, Clone)]
pub struct RustObject {
    string: String,
}

impl RustObject {
    fn new() -> Self {
        Self { string: "".into() }
    }

    fn from_string(string: String) -> Self {
        Self { string }
    }

    fn identity_string(&self, s: String) -> String {
        s
    }

    fn get_string(&self) -> String {
        self.string.clone()
    }

    fn length(&self) -> i32 {
        self.string.len().try_into().unwrap()
    }
}

include!(concat!(env!("OUT_DIR"), "/decorators.uniffi.rs"));
