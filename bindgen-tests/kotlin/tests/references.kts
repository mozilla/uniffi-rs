/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_bindgen_tests.*

assert(roundtripU8Ref(2.toUByte()) == 2.toUByte())

val i = ReferenceTestInterface()
assert(i.doubleValue(2u) == 4u)
assert(callDoubleValue(i, 3u) == 6u)

val t = createReferenceTestTraitInterface()
assert(callTripleValueTraitInterface(t, 10u) == 30u)
