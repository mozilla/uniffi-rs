/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This test ensures the generated code works as expected when documentation comments are generated.
// Note: we do not check for existence of the doc comments here, as they are not programmatically
// exposed to the code.
// https://github.com/mozilla/uniffi-rs/pull/1493#discussion_r1375337478

import uniffi_docstring_proc_macro

test()

var _ = EnumTest.one
var _ = EnumTest.two

var _ = AssociatedEnumTest.test(code: 0)
var _ = AssociatedEnumTest.test2(code: 0)

var _ = ErrorTest.One(message: "hello")
var _ = ErrorTest.Two(message: "hello")

var _ = AssociatedErrorTest.Test(code: 0)
var _ = AssociatedErrorTest.Test2(code: 0)

var obj1 = ObjectTest()
var obj2 = ObjectTest.newAlternate()
obj2.test()

var rec = RecordTest(test: 123)
var recField = rec.test

class CallbackImpls: CallbackTest {
    func test() {}
}

