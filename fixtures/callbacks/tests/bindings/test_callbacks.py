# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from fixture_callbacks import *
import unittest

# A bit more systematic in testing, but this time in English.
#
# 1. Pass in the callback as arguments.
# Make the callback methods use multiple arguments, with a variety of types, and
# with a variety of return types.
rust_getters = RustGetters()

class PythonGetters(ForeignGetters):
    def get_bool(self, v, argumentTwo):
        return v ^ argumentTwo

    def get_string(self, v, arg2):
        if v == "bad-argument":
            raise SimpleError.BadArgument()
        elif v == "unexpected-error":
            raise ValueError("unexpected value")
        if arg2:
            return "1234567890123"
        else:
            return v

    def get_option(self, v, arg2):
        if v == "bad-argument":
            raise ComplexError.ReallyBadArgument(20)
        elif v == "unexpected-error":
            raise ValueError("unexpected value")
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

    def get_nothing(self, v):
        if v == "bad-argument":
            raise SimpleError.BadArgument()
        elif v == "unexpected-error":
            raise ValueError("unexpected value")

class ForeignGettersTest(unittest.TestCase):
    def test_get_bool(self):
        callback = PythonGetters()
        for v in [True, False]:
            flag = True
            expected = callback.get_bool(v, flag)
            observed = rust_getters.get_bool(callback, v, flag)
            self.assertEqual(expected, observed, f"roundtripping through callback: {expected} != {observed}")

    def test_get_list(self):
        callback = PythonGetters()
        for v in [[1, 2], [0, 1]]:
            flag = True
            expected = callback.get_list(v, flag)
            observed = rust_getters.get_list(callback, v, flag)
            self.assertEqual(expected, observed, f"roundtripping through callback: {expected} != {observed}")

    def test_get_string(self):
        callback = PythonGetters()
        for v in ["Hello", "world"]:
            flag = True
            expected = callback.get_string(v, flag)
            observed = rust_getters.get_string(callback, v, flag)
            self.assertEqual(expected, observed, f"roundtripping through callback: {expected} != {observed}")

    def test_get_optional(self):
        callback = PythonGetters()
        for v in ["Some", None]:
            flag = False
            expected = callback.get_option(v, flag)
            observed = rust_getters.get_option(callback, v, flag)
            self.assertEqual(expected, observed, f"roundtripping through callback: {expected} != {observed}")

    def test_get_string_optional_callback(self):
        self.assertEqual(rust_getters.get_string_optional_callback(PythonGetters(), "TestString", False), "TestString")
        self.assertEqual(rust_getters.get_string_optional_callback(None, "TestString", False), None)

    def test_get_nothing(self):
        rust_getters.get_nothing(PythonGetters(), "TestString")

# 2. Pass the callback in as a constructor argument, to be stored on the Object struct.
# This is crucial if we want to configure a system at startup,
# then use it without passing callbacks all the time.

class StoredPythonStringifier(StoredForeignStringifier):
    def from_simple_type(self, value):
        return f"python: {value}"

class StoredStringifierTest(unittest.TestCase):
    def test_stored_stringifier(self):
        python_stringifier = StoredPythonStringifier()
        rust_stringifier = RustStringifier(python_stringifier)
        for v in [1, 2]:
            expected = python_stringifier.from_simple_type(v)
            observed = rust_stringifier.from_simple_type(v)
            self.assertEqual(expected, observed, f"callback is sent on construction: {expected} != {observed}")

class TestCallbackErrors(unittest.TestCase):
    def test_simple_errors(self):
        callback = PythonGetters()
        with self.assertRaises(SimpleError.BadArgument):
            rust_getters.get_string(callback, "bad-argument", True)
        with self.assertRaises(SimpleError.UnexpectedError):
            rust_getters.get_string(callback, "unexpected-error", True)

        with self.assertRaises(SimpleError.BadArgument):
            rust_getters.get_nothing(callback, "bad-argument")
        with self.assertRaises(SimpleError.UnexpectedError):
            rust_getters.get_nothing(callback, "unexpected-error")

    def test_complex_errors(self):
        callback = PythonGetters()
        with self.assertRaises(ComplexError.ReallyBadArgument) as cm:
            rust_getters.get_option(callback, "bad-argument", True)
        self.assertEqual(cm.exception.code, 20)
        with self.assertRaises(ComplexError.UnexpectedErrorWithReason) as cm:
            rust_getters.get_option(callback, "unexpected-error", True)
        self.assertEqual(cm.exception.reason, repr(ValueError("unexpected value")))

class TestCallbackLifetime(unittest.TestCase):
    def test_callback_reference_does_not_invalidate_other_references(self):
        # `stringifier` must remain valid after `rust_stringifier_2` drops the reference
        stringifier = StoredPythonStringifier()
        rust_stringifier_1 = RustStringifier(stringifier)
        rust_stringifier_2 = RustStringifier(stringifier)
        assert("python: 123" == rust_stringifier_2.from_simple_type(123))
        del rust_stringifier_2
        assert("python: 123" == rust_stringifier_1.from_simple_type(123))

unittest.main()
