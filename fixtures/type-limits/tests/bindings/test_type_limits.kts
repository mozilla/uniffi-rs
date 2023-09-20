/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_type_limits.*;

// test_strings
try {
    takeString("\ud800")
    throw RuntimeException("Should have thrown an CharacterCodingException exception!")
} catch (e: java.nio.charset.CharacterCodingException) {
    // It's okay!
}
assert(takeString("") == "")
assert(takeString("愛") == "愛")
assert(takeString("💖") == "💖")
