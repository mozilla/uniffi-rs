/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// CustomType1 doesn't have any Swift custom type associated with it
assert(roundtripCustomType1(custom1: 100) == 100)

// CustomType2 is a dict in swift.  See `uniffi.toml` for the configuration that does this.
assert(roundtripCustomType2(custom2: ["value": 200]) == ["value": 200])

