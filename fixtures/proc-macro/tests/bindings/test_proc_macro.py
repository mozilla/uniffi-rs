# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_proc_macro import *

one = make_one(123)
assert one.inner == 123

two = Two("a")
assert take_two(two) == "a"

rwb = RecordWithBytes(bytes([1,2,3]))
assert take_record_with_bytes(rwb) == bytes([1,2,3])

obj = Object()
obj = Object.named_ctor(1)
assert obj.is_heavy() == MaybeBool.UNCERTAIN

trait_impl = obj.get_trait(None)
assert trait_impl.name() == "TraitImpl"
assert obj.get_trait(trait_impl).name() == "TraitImpl"

assert enum_identity(MaybeBool.TRUE) == MaybeBool.TRUE

# just make sure this works / doesn't crash
three = Three(obj)

assert(make_zero().inner == "ZERO")
assert(make_record_with_bytes().some_bytes == bytes([0, 1, 2, 3, 4]))

try:
    always_fails()
except BasicError.OsError:
    pass
else:
    raise Exception("always_fails should have thrown")

obj.do_stuff(5)

try:
    obj.do_stuff(0)
except FlatError.InvalidInput:
    pass
else:
    raise Exception("do_stuff should throw if its argument is 0")

class PyTestCallbackInterface(TestCallbackInterface):
    def do_nothing(self):
        pass

    def add(self, a, b):
        return a + b

    def optional(self, a):
        if a is None:
            return 0
        return a

    def with_bytes(self, rwb):
        return rwb.some_bytes

    def try_parse_int(self, value):
        if value == "force-unexpected-error":
            # raise an error that's not expected
            raise KeyError(value)
        try:
            return int(value)
        except BaseException:
            raise BasicError.InvalidInput()

    def callback_handler(self, h):
        v = h.take_error(BasicError.InvalidInput())
        return v

test_callback_interface(PyTestCallbackInterface())
