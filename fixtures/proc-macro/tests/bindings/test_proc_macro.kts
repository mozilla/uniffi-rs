/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.proc_macro.*;

val one = makeOne(123)
assert(one.inner == 123)

val two = Two("a", null)
assert(takeTwo(two) == "a")

val obj = Object()
assert(obj.isHeavy() == MaybeBool.UNCERTAIN)

assert(enumIdentity(MaybeBool.TRUE) == MaybeBool.TRUE)

// just make sure this works / doesn't crash
val three = Three(obj)

assert(makeZero().inner == "ZERO")

try {
    alwaysFails()
    throw RuntimeException("alwaysFails should have thrown")
} catch (e: BasicException) {
}

obj.doStuff(5u)

try {
    obj.doStuff(0u)
    throw RuntimeException("doStuff should throw if its argument is 0")
} catch (e: FlatException) {
}
