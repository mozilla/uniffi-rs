# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_proc_macro import *

one = make_one(123)
assert one.inner == 123

two = Two("a", None)
assert take_two(two) == "a"

obj = make_object()
assert obj.is_heavy() == MaybeBool.UNCERTAIN

assert enum_identity(MaybeBool.TRUE) == MaybeBool.TRUE

# just make sure this works / doesn't crash
three = Three(obj)

try:
    always_fails()
except BasicError.OsError:
    pass
else:
    raise Exception("always_fails should have thrown")

obj.do_stuff(5)

try:
    obj.do_stuff(0)
except BasicError.InvalidInput:
    pass
else:
    raise Exception("do_stuff should throw if its argument is 0")
