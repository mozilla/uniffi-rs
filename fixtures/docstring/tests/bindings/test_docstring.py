# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Test namespace
import uniffi_docstring
assert uniffi_docstring.__doc__

from uniffi_docstring import *

# Test function
assert test.__doc__
val = test()

# Test enums
assert EnumTest.__doc__
val = EnumTest.ONE
val = EnumTest.TWO

assert AssociatedEnumTest.__doc__
val = AssociatedEnumTest.TEST(0)
val = AssociatedEnumTest.TEST2(0)

# Test errors
assert ErrorTest.__doc__
val = ErrorTest.One("hello")
val = ErrorTest.Two("hello")

assert AssociatedErrorTest.__doc__
val = AssociatedErrorTest.Test(0)
val = AssociatedErrorTest.Test2(0)

# Test objects
assert ObjectTest.__doc__
val = ObjectTest

assert ObjectTest.__doc__
val = ObjectTest.new_alternate()

assert val.test.__doc__
val.test()

# Test records
assert RecordTest.__doc__
val = RecordTest(123)

assert val.test.__doc__
val = val.test

# Test callbacks
assert CallbackTest.__doc__
class CallbackImpls(CallbackTest):
    def test():
        pass
