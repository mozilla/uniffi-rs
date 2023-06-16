/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_type_limits.*;

// test_strings
assert(takeString("\ud800") == "?") // surrogate
assert(takeString("") == "")
assert(takeString("愛") == "愛")
assert(takeString("💖") == "💖")
