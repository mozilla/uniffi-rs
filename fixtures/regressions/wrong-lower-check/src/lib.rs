/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn optional_string(s: Option<String>) -> Option<String> {
    s
}

struct Klass;

impl Klass {
    fn new() -> Self {
        Self
    }

    fn optional_string(&self, value: Option<String>) -> Option<String> {
        value
    }
}

uniffi::include_scaffolding!("test");
