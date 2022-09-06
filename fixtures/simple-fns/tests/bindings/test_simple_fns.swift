/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_simple_fns

assert(getString() == "String created by Rust")
assert(getInt() == 1289)
assert(stringIdentity(s: "String created by Kotlin") == "String created by Kotlin")
assert(byteToU32(byte: 255) == 255)

let aSet = newSet()
addToSet(set: aSet, value: "foo")
addToSet(set: aSet, value: "bar")
assert(setContains(set: aSet, value: "foo"))
assert(setContains(set: aSet, value: "bar"))
assert(!setContains(set: aSet, value: "baz"))
