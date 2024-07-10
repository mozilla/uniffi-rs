# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
import ctypes
from datetime import datetime, timezone

from coverall import *

class TestCoverall(unittest.TestCase):
    # Any test not terminating with zero objects alive will cause others to
    # fail - this helps us work out which test kept things alive.
    def tearDown(self):
        self.assertEqual(get_num_alive(), 0)

    def test_some_dict(self):
        d = create_some_dict()
        self.assertEqual(d.text, "text")
        self.assertEqual(d.maybe_text, "maybe_text")
        self.assertEqual(d.some_bytes, b"some_bytes")
        self.assertEqual(d.maybe_some_bytes, b"maybe_some_bytes")
        self.assertTrue(d.a_bool)
        self.assertFalse(d.maybe_a_bool)
        self.assertEqual(d.unsigned8, 1)
        self.assertEqual(d.maybe_unsigned8, 2)
        self.assertEqual(d.unsigned16, 3)
        self.assertEqual(d.maybe_unsigned16, 4)
        self.assertEqual(d.unsigned64, 18446744073709551615)
        self.assertEqual(d.maybe_unsigned64, 0)
        self.assertEqual(d.signed8, 8)
        self.assertEqual(d.maybe_signed8, 0)
        self.assertEqual(d.signed64, 9223372036854775807)
        self.assertEqual(d.maybe_signed64, 0)
        self.assertEqual(d.coveralls.get_name(), "some_dict")
        self.assertEqual(d.test_trait.name(), "node-2")

        # floats should be "close enough" - although it's mildly surprising that
        # we need to specify `places=6` whereas the default is 7.
        self.assertAlmostEqual(d.float32, 1.2345, places=6)
        self.assertAlmostEqual(d.maybe_float32, 22.0/7.0, places=6)
        self.assertAlmostEqual(d.float64, 0.0)
        self.assertAlmostEqual(d.maybe_float64, 1.0)

    def test_none_dict(self):
        d = create_none_dict()
        self.assertEqual(d.text, "text")
        self.assertIsNone(d.maybe_text)
        self.assertEqual(d.some_bytes, b"some_bytes")
        self.assertIsNone(d.maybe_some_bytes)
        self.assertTrue(d.a_bool)
        self.assertIsNone(d.maybe_a_bool)
        self.assertEqual(d.unsigned8, 1)
        self.assertIsNone(d.maybe_unsigned8)
        self.assertEqual(d.unsigned16, 3)
        self.assertIsNone(d.maybe_unsigned16)
        self.assertEqual(d.unsigned64, 18446744073709551615)
        self.assertIsNone(d.maybe_unsigned64)
        self.assertEqual(d.signed8, 8)
        self.assertIsNone(d.maybe_signed8)
        self.assertEqual(d.signed64, 9223372036854775807)
        self.assertIsNone(d.maybe_signed64)

        self.assertAlmostEqual(d.float32, 1.2345, places=6)
        self.assertIsNone(d.maybe_float32)
        self.assertAlmostEqual(d.float64, 0.0)
        self.assertIsNone(d.maybe_float64)
        self.assertIsNone(d.coveralls)
        self.assertIsNone(d.test_trait)

    def test_constructors(self):
        self.assertEqual(get_num_alive(), 0)
        # must work.
        coveralls = Coveralls("c1")
        self.assertEqual(get_num_alive(), 1)
        # make sure it really is our Coveralls object.
        self.assertEqual(coveralls.get_name(), "c1")
        # must also work.
        coveralls2 = Coveralls.fallible_new("c2", False)
        self.assertEqual(get_num_alive(), 2)
        # make sure it really is our Coveralls object.
        self.assertEqual(coveralls2.get_name(), "c2")

        with self.assertRaises(CoverallError.TooManyHoles):
            Coveralls.fallible_new("", True)

        with self.assertRaisesRegex(InternalError, "expected panic: woe is me"):
            Coveralls.panicking_new("expected panic: woe is me")

        # in the absence of cycles Python is deterministic in killing refs
        coveralls2 = None
        self.assertEqual(get_num_alive(), 1)
        coveralls = None
        self.assertEqual(get_num_alive(), 0)


    def test_simple_errors(self):
        # This is testing enums which have been described in UDL via `enum` or via
        # procmacros with `#[uniffi(flat_error)]`. Whether the variants have fields or not
        # in Rust, these are treated as though each variant uses a single string.
        coveralls = Coveralls("test_errors")
        self.assertEqual(coveralls.get_name(), "test_errors")

        with self.assertRaisesRegex(CoverallError.TooManyHoles, "The coverall has too many holes") as cm:
            coveralls.maybe_throw(True)
        self.assertEqual(len(cm.exception.args), 1)
        self.assertEqual(type(cm.exception.args[0]), str)
        self.assertEqual(str(cm.exception), "The coverall has too many holes")
        self.assertEqual(repr(cm.exception), "CoverallError.TooManyHoles('The coverall has too many holes')")

        with self.assertRaisesRegex(CoverallMacroError.TooManyMacros, "The coverall has too many macros") as cm:
            throw_macro_error()
        self.assertEqual(len(cm.exception.args), 1)
        self.assertEqual(type(cm.exception.args[0]), str)
        self.assertEqual(str(cm.exception), "The coverall has too many macros")
        self.assertEqual(repr(cm.exception), "CoverallMacroError.TooManyMacros('The coverall has too many macros')")

        with self.assertRaises(CoverallError.TooManyHoles) as cm:
            coveralls.maybe_throw_into(True)
        self.assertEqual(len(cm.exception.args), 1)
        self.assertEqual(type(cm.exception.args[0]), str)

        with self.assertRaisesRegex(InternalError, "expected panic: oh no"):
            coveralls.panic("expected panic: oh no")

    def test_flat_errors(self):
        # This is testing enums which have fields in Rust but are marked as "flat" for the ffi.
        with self.assertRaisesRegex(CoverallFlatError.TooManyVariants, "Too many variants: 99") as cm:
            throw_flat_error()
        self.assertEqual(len(cm.exception.args), 1)
        self.assertEqual(type(cm.exception.args[0]), str)

        with self.assertRaisesRegex(CoverallFlatMacroError.TooManyVariants, "Too many variants: 88") as cm:
            throw_flat_macro_error()
        self.assertEqual(len(cm.exception.args), 1)
        self.assertEqual(type(cm.exception.args[0]), str)

        # CoverallRichErrorNoVariantData is "flat" on the Rust side, but because it was
        # described in the UDL via `[Error]interface`, it doesn't get the "flat" (ie, lowered as though
        # each variant had a simple string) semantics.
        with self.assertRaises(CoverallRichErrorNoVariantData.TooManyPlainVariants) as cm:
            throw_rich_error_no_variant_data()
        self.assertEqual(len(cm.exception.args), 0)
        self.assertEqual(str(cm.exception), "") # probably not ideal!
        self.assertEqual(repr(cm.exception), "CoverallRichErrorNoVariantData.TooManyPlainVariants()")

    def test_complex_errors(self):
        # This is testing fields with variants which are exposed via the FFI.
        coveralls = Coveralls("test_complex_errors")

        # Test success
        self.assertEqual(True, coveralls.maybe_throw_complex(0))

        # Test errors
        with self.assertRaises(ComplexError.OsError) as cm:
            coveralls.maybe_throw_complex(1)
        self.assertEqual(cm.exception.code, 10)
        self.assertEqual(cm.exception.extended_code, 20)
        self.assertEqual(str(cm.exception), "code=10, extended_code=20")
        self.assertEqual(repr(cm.exception), "ComplexError.OsError(code=10, extended_code=20)")

        with self.assertRaises(ComplexMacroError.OsError) as cm:
            throw_complex_macro_error()
        self.assertEqual(cm.exception.code, 1)
        self.assertEqual(cm.exception.extended_code, 2)
        self.assertEqual(str(cm.exception), "code=1, extended_code=2")
        self.assertEqual(repr(cm.exception), "ComplexMacroError.OsError(code=1, extended_code=2)")

        with self.assertRaises(ComplexError.PermissionDenied) as cm:
            coveralls.maybe_throw_complex(2)
        self.assertEqual(cm.exception.reason, "Forbidden")
        self.assertEqual(str(cm.exception), "reason='Forbidden'")
        self.assertEqual(repr(cm.exception), "ComplexError.PermissionDenied(reason='Forbidden')")

        with self.assertRaises(ComplexError.UnknownError) as cm:
            coveralls.maybe_throw_complex(3)
        self.assertEqual(str(cm.exception), "")
        self.assertEqual(repr(cm.exception), "ComplexError.UnknownError()")

        # Test panics, which should cause InternalError to be raised
        with self.assertRaises(InternalError) as cm:
            coveralls.maybe_throw_complex(4)

    def test_error_values(self):
        with self.assertRaises(RootError.Complex) as cm:
            throw_root_error()
        self.assertEqual(cm.exception.error.code, 1)

        e = get_root_error()
        # Renamed `OtherError` to `AnotherError` via `uniffi.toml`
        self.assertEqual(e.error, AnotherError.UNEXPECTED)

        self.assertTrue(isinstance(get_complex_error(None), ComplexError.PermissionDenied))
        self.assertIsNone(get_error_dict(None).complex_error)

    def test_enums(self):
        e = get_simple_flat_macro_enum(0)
        self.assertTrue(isinstance(e, SimpleFlatMacroEnum.FIRST))

    def test_self_by_arc(self):
        coveralls = Coveralls("test_self_by_arc")
        # One reference is held by the handlemap, and one by the `Arc<Self>` method receiver.
        self.assertEqual(coveralls.strong_count(), 2)

    def test_arcs(self):
        coveralls = Coveralls("test_arcs")
        self.assertEqual(get_num_alive(), 1)
        self.assertEqual(coveralls.strong_count(), 2)
        self.assertIsNone(coveralls.get_other())
        coveralls.take_other(coveralls)
        # should now be a new strong ref.
        self.assertEqual(coveralls.strong_count(), 3)
        # but the same number of instances.
        self.assertEqual(get_num_alive(), 1)
        # and check it's the correct object.
        self.assertEqual(coveralls.get_other().get_name(), "test_arcs")

        with self.assertRaises(CoverallError.TooManyHoles):
            coveralls.take_other_fallible()

        with self.assertRaisesRegex(InternalError, "expected panic: with an arc!"):
            coveralls.take_other_panic("expected panic: with an arc!")

        coveralls.take_other(None)
        self.assertEqual(coveralls.strong_count(), 2)
        coveralls = None
        self.assertEqual(get_num_alive(), 0)

    def test_return_objects(self):
        coveralls = Coveralls("test_return_objects")
        self.assertEqual(get_num_alive(), 1)
        self.assertEqual(coveralls.strong_count(), 2)
        c2 = coveralls.clone_me()
        self.assertEqual(c2.get_name(), coveralls.get_name())
        self.assertEqual(get_num_alive(), 2)
        self.assertEqual(c2.strong_count(), 2)

        coveralls.take_other(c2)
        # same number alive but `c2` has an additional ref count.
        self.assertEqual(get_num_alive(), 2)
        self.assertEqual(coveralls.strong_count(), 2)
        self.assertEqual(c2.strong_count(), 3)

        # We can drop Python's reference to `c2`, but the rust struct will not
        # be dropped as coveralls hold an `Arc<>` to it.
        c2 = None
        self.assertEqual(get_num_alive(), 2)

        coveralls.add_patch(Patch(Color.RED))
        coveralls.add_repair(
            Repair(when=datetime.now(timezone.utc), patch=Patch(Color.BLUE))
        )
        self.assertEqual(len(coveralls.get_repairs()), 2)

        # Dropping `coveralls` will kill both.
        coveralls = None
        self.assertEqual(get_num_alive(), 0)

    def test_throwing_constructor(self):
        with self.assertRaises(CoverallError.TooManyHoles):
            FalliblePatch()
        with self.assertRaises(CoverallError.TooManyHoles):
            FalliblePatch.secondary()

    def test_bad_objects(self):
        coveralls = Coveralls("test_bad_objects")
        patch = Patch(Color.RED)
        # `coveralls.take_other` wants `Coveralls` not `Patch`
        with self.assertRaisesRegex(TypeError, "Coveralls.*Patch"):
            coveralls.take_other(patch)

    def test_dict_with_defaults(self):
        """ This does not call Rust code. """

        d = DictWithDefaults()
        self.assertEqual("default-value", d.name)
        self.assertEqual(None, d.category)
        self.assertEqual(31, d.integer)

        d = DictWithDefaults(name="this", category="that", integer=42)
        self.assertEqual("this", d.name)
        self.assertEqual("that", d.category)
        self.assertEqual(42, d.integer)

    def test_dict_with_non_string_keys(self):
        coveralls = Coveralls("test_dict")

        dict1 = coveralls.get_dict(key="answer", value=42)
        assert dict1["answer"] == 42

        dict2 = coveralls.get_dict2(key="answer", value=42)
        assert dict2["answer"] == 42

        dict3 = coveralls.get_dict3(key=31, value=42)
        assert dict3[31] == 42

    def test_bytes(self):
        coveralls = Coveralls("test_bytes")
        self.assertEqual(coveralls.reverse(b"123"), b"321")

    def test_return_only_dict(self):
        # try_input_return_only_dict can never work, since ReturnOnlyDict should only be returned
        # from Rust not inputted.  Test that an attempt raises an internal error rather than tries
        # to use an invalid value.
        with self.assertRaises(InternalError):
            try_input_return_only_dict(ReturnOnlyDict(e=CoverallFlatError.TooManyVariants))

class PyGetters:
    def get_bool(self, v, arg2):
        return v ^ arg2

    def get_string(self, v, arg2):
        if v == "too-many-holes":
            raise CoverallError.TooManyHoles
        elif v == "unexpected-error":
            raise RuntimeError("unexpected error")
        elif arg2:
            return v.upper()
        else:
            return v

    def get_option(self, v, arg2):
        if v == "os-error":
            raise ComplexError.OsError(100, 200)
        elif v == "unknown-error":
            raise ComplexError.UnknownError
        elif arg2:
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

    def get_nothing(self, _v):
        return None

    def round_trip_object(self, coveralls):
        return coveralls

class PyNode:
    def __init__(self):
        self.parent = None

    def name(self):
        return "node-py"

    def set_parent(self, parent):
        self.parent = parent

    def get_parent(self):
        return self.parent

    def strong_count(self):
        return 0 # TODO

class TraitsTest(unittest.TestCase):
    # Test traits implemented in Rust
    # def test_rust_getters(self):
    #     test_getters(None)
    #     self.check_getters_from_python(make_rust_getters())

    # Test traits implemented in Rust
    def test_python_getters(self):
        test_getters(PyGetters())
        #self.check_getters_from_python(PyGetters())

    def check_getters_from_python(self, getters):
        self.assertEqual(getters.get_bool(True, True), False);
        self.assertEqual(getters.get_bool(True, False), True);
        self.assertEqual(getters.get_bool(False, True), True);
        self.assertEqual(getters.get_bool(False, False), False);

        self.assertEqual(getters.get_string("hello", False), "hello");
        self.assertEqual(getters.get_string("hello", True), "HELLO");

        self.assertEqual(getters.get_option("hello", True), "HELLO");
        self.assertEqual(getters.get_option("hello", False), "hello");
        self.assertEqual(getters.get_option("", True), None);

        self.assertEqual(getters.get_list([1, 2, 3], True), [1, 2, 3]);
        self.assertEqual(getters.get_list([1, 2, 3], False), [])

        self.assertEqual(getters.get_nothing("hello"), None);

        with self.assertRaises(CoverallError.TooManyHoles):
            getters.get_string("too-many-holes", True)

        with self.assertRaises(ComplexError.OsError) as cm:
            getters.get_option("os-error", True)
        self.assertEqual(cm.exception.code, 100)
        self.assertEqual(cm.exception.extended_code, 200)

        with self.assertRaises(ComplexError.UnknownError):
            getters.get_option("unknown-error", True)

        with self.assertRaises(InternalError):
            getters.get_string("unexpected-error", True)

    def test_path(self):
        # Get traits creates 2 objects that implement the trait
        traits = get_traits()
        self.assertEqual(traits[0].name(), "node-1")
        # Note: strong counts are 1 more than you might expect, because the strong_count() method
        # holds a strong ref.
        self.assertEqual(traits[0].strong_count(), 2)

        self.assertEqual(traits[1].name(), "node-2")
        self.assertEqual(traits[1].strong_count(), 2)

        # Let's try connecting them together
        traits[0].set_parent(traits[1])
        # Note: this doesn't increase the Rust strong count, since we wrap the Rust impl with a
        # python impl before passing it to `set_parent()`
        self.assertEqual(traits[1].strong_count(), 2)
        self.assertEqual(ancestor_names(traits[0]), ["node-2"])
        self.assertEqual(ancestor_names(traits[1]), [])
        self.assertEqual(traits[0].get_parent().name(), "node-2")

        # Throw in a Python implementation of the trait
        # The ancestry chain now goes traits[0] -> traits[1] -> py_node
        py_node = PyNode()
        traits[1].set_parent(py_node)
        self.assertEqual(ancestor_names(traits[0]), ["node-2", "node-py"])
        self.assertEqual(ancestor_names(traits[1]), ["node-py"])
        self.assertEqual(ancestor_names(py_node), [])

        # Rotating things.
        # The ancestry chain now goes py_node -> traits[0] -> traits[1]
        traits[1].set_parent(None)
        py_node.set_parent(traits[0])
        self.assertEqual(ancestor_names(py_node), ["node-1", "node-2"])
        self.assertEqual(ancestor_names(traits[0]), ["node-2"])
        self.assertEqual(ancestor_names(traits[1]), [])

        # Make sure we don't crash when undoing it all
        py_node.set_parent(None)
        traits[0].set_parent(None)

    def test_round_tripping(self):
        rust_getters = make_rust_getters();
        coveralls = Coveralls("test_round_tripping")
        # Check that these don't cause use-after-free bugs
        test_round_trip_through_rust(rust_getters)

        test_round_trip_through_foreign(PyGetters())

    def test_rust_only_traits(self):
        traits = get_string_util_traits()
        self.assertEqual(traits[0].concat("cow", "boy"), "cowboy")
        self.assertEqual(traits[1].concat("cow", "boy"), "cowboy")


# class TestRenaming(unittest.TestCase):
#     def test_function_renaming(self):
#         # Test if the old function name is not available
#         with self.assertRaises(NameError):
#             SayHello()  # This should raise NameError since SayHello was renamed to SayHi
#
#         with self.assertRaises(ValueError):
#             SayHi()  # This should raise ValueError since SayHi doesn't have a default constructor

if __name__=='__main__':
    unittest.main()
