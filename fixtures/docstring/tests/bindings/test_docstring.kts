/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.docstring.*;

test()

EnumTest.ONE

AssociatedEnumTest.Test(0)

ErrorTest.One("hello")

AssociatedErrorTest.Test(0)

val obj1 = ObjectTest
val obj2 = ObjectTest.newAlternate()
obj2.test()

val rec = RecordTest(123)
val recField = rec.test

class CallbackImpls() : CallbackTest {
    override fun test() {}
}
