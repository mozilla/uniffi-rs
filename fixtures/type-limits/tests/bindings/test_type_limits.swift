/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_type_limits

// test_strings
do {
    // strings cannot contain surrogates, "\u{d800}" gives an error.
    assert(takeString(v: "") == "")
    assert(takeString(v: "愛") == "愛")
    assert(takeString(v: "💖") == "💖")
}
