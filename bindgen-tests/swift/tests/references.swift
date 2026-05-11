/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripU8Ref(a: 2) == 2)

let interface = ReferenceTestInterface()
assert(interface.doubleValue(a: 2) == 4)
assert(callDoubleValue(i: interface, a: 3) == 6)

let traitInterface = createReferenceTestTraitInterface()
assert(callTripleValueTraitInterface(t: traitInterface, a: 10) == 30)
