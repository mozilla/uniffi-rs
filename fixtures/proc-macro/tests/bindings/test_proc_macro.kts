/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.proc_macro.*;

val one = makeOne(123)
assert(one.inner == 123)

val two = Two("a", null)
assert(takeTwo(two) == "a")

// just make sure this works / doesn't crash
val three = Three(makeObject())
