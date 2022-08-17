# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from callbacks import *

# A bit more systematic in testing, but this time in English.
#
# 1. Pass in the callback as arguments.
# Make the callback methods use multiple aruments, with a variety of types, and
# with a variety of return types.
rust_getters = RustGetters()

class PythonGetters(ForeignGetters):
    def get_bool(self, v, argumentTwo):
        return v ^ argumentTwo

    def get_string(self, v, arg2):
        if arg2:
            return "1234567890123"
        else:
            return v

    def get_option(self, v, arg2):
        if arg2:
            if v:
                return v.upper()
            else:
                return None
        else:
            return v

    def get_list(self, v, arg2):
        if arg2:
            return v
        else:
            return []

callback = PythonGetters()
for v in [True, False]:
    flag = True
    expected = callback.get_bool(v, flag)
    observed = rust_getters.get_bool(callback, v, flag)
    assert expected == observed, f"roundtripping through callback: {expected} != {observed}"

for v in [[1, 2], [0, 1]]:
    flag = True
    expected = callback.get_list(v, flag)
    observed = rust_getters.get_list(callback, v, flag)
    assert expected == observed, f"roundtripping through callback: {expected} != {observed}"

for v in ["Hello", "world"]:
    flag = True
    expected = callback.get_string(v, flag)
    observed = rust_getters.get_string(callback, v, flag)
    assert expected == observed, f"roundtripping through callback: {expected} != {observed}"

for v in ["Some", None]:
    flag = False
    expected = callback.get_option(v, flag)
    observed = rust_getters.get_option(callback, v, flag)
    assert expected == observed, f"roundtripping through callback: {expected} != {observed}"


assert rust_getters.get_string_optional_callback(callback, "TestString", False) == "TestString"
assert rust_getters.get_string_optional_callback(None, "TestString", False) == None

# 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
# This is crucial if we want to configure a system at startup,
# then use it without passing callbacks all the time.

class StoredPythonStringifier(StoredForeignStringifier):
    def from_simple_type(self, value):
        return f"python: {value}"

    # We don't test this, but we're checking that the arg type is included in the minimal list of types used
    # in the UDL.
    # If this doesn't compile, then look at TypeResolver.
    def from_complex_type(values):
        return f"python: {values}"

python_stringifier = StoredPythonStringifier()
rust_stringifier = RustStringifier(python_stringifier)
for v in [1, 2]:
    expected = python_stringifier.from_simple_type(v)
    observed = rust_stringifier.from_simple_type(v)
    assert expected == observed, f"callback is sent on construction: {expected} != {observed}"
