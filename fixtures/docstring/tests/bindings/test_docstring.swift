/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_docstring

test()

var _ = EnumTest.one

var _ = AssociatedEnumTest.test

var _ = ErrorTest.One(message: "hello")

var _ = AssociatedErrorTest.Test

var obj1 = ObjectTest()
var obj2 = ObjectTest.newAlternate()
obj2.test()

var rec = RecordTest(test: 123)
var recField = rec.test

class CallbackImpls: CallbackTest {
    func test() {}
}

