# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
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
            Coveralls.panicing_new("expected panic: woe is me")

        # in the absence of cycles Python is deterministic in killing refs
        coveralls2 = None
        self.assertEqual(get_num_alive(), 1)
        coveralls = None
        self.assertEqual(get_num_alive(), 0)

        glob = Coveralls.global_new()
        self.assertEqual(glob.get_name(), "global")
        # We can also get it via global_coveralls
        self.assertEqual(global_coveralls().get_name(), "global")

    def test_simple_errors(self):
        coveralls = Coveralls("test_errors")
        self.assertEqual(coveralls.get_name(), "test_errors")

        with self.assertRaisesRegex(CoverallError.TooManyHoles, "The coverall has too many holes"):
            coveralls.maybe_throw(True)

        with self.assertRaises(CoverallError.TooManyHoles):
            coveralls.maybe_throw_into(True)

        with self.assertRaisesRegex(InternalError, "expected panic: oh no"):
            coveralls.panic("expected panic: oh no")

    def test_complex_errors(self):
        coveralls = Coveralls("test_complex_errors")

        # Test success
        self.assertEqual(True, coveralls.maybe_throw_complex(0))

        # Test errors
        with self.assertRaises(ComplexError.OsError) as cm:
            coveralls.maybe_throw_complex(1)
        self.assertEqual(cm.exception.code, 10)
        self.assertEqual(cm.exception.extended_code, 20)
        self.assertEqual(str(cm.exception), "ComplexError.OsError(code=10, extended_code=20)")

        with self.assertRaises(ComplexError.PermissionDenied) as cm:
            coveralls.maybe_throw_complex(2)
        self.assertEqual(cm.exception.reason, "Forbidden")
        self.assertEqual(str(cm.exception), "ComplexError.PermissionDenied(reason='Forbidden')")

        # Test panics, which should cause InternalError to be raised
        with self.assertRaises(InternalError) as cm:
            coveralls.maybe_throw_complex(3)

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

        # Dropping `coveralls` will kill both.
        coveralls = None
        self.assertEqual(get_num_alive(), 0)

    def test_bad_objects(self):
        coveralls = Coveralls("test_bad_objects")
        patch = Patch(Color.RED)
        # `coveralls.take_other` wants `Coveralls` not `Patch`
        with self.assertRaisesRegex(TypeError, "Coveralls.*Patch"):
            coveralls.take_other(patch)

if __name__=='__main__':
    unittest.main()
