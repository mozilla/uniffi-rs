/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This test ensures the generated code works as expected when documentation comments are generated.
// Note: we do not check for existence of the doc comments here, as they are not programmatically
// exposed to the code.
// https://github.com/mozilla/uniffi-rs/pull/1493#discussion_r1375337478

import uniffi.fixture.docstring.*;

test()
test_multiline()

EnumTest.ONE
EnumTest.TWO

AssociatedEnumTest.Test(0)
AssociatedEnumTest.Test2(0)

ErrorTest.One("hello")
ErrorTest.Two("hello")

AssociatedErrorTest.Test(0)
AssociatedErrorTest.Test2(0)

val obj1 = ObjectTest
val obj2 = ObjectTest.newAlternate()
obj2.test()

val rec = RecordTest(123)
val recField = rec.test

class CallbackImpls() : CallbackTest {
    override fun test() {}
}
