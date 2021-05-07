# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from coverall import *

class TestCoverall(unittest.TestCase):
    def test_some_dict(self):
        d = create_some_dict()
        self.assertEqual(d.text, "text")
        self.assertEqual(d.maybe_text, "maybe_text")
        self.assertTrue(d.a_bool)
        self.assertFalse(d.maybe_a_bool)
        self.assertEqual(d.unsigned8, 1)
        self.assertEqual(d.maybe_unsigned8, 2)
        self.assertEqual(d.unsigned64, 18446744073709551615)
        self.assertEqual(d.maybe_unsigned64, 0)
        self.assertEqual(d.signed8, 8)
        self.assertEqual(d.maybe_signed8, 0)
        self.assertEqual(d.signed64, 9223372036854775807)
        self.assertEqual(d.maybe_signed64, 0)

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

        # in the absence of cycles Python is deterministic killing refs
        coveralls2 = None
        self.assertEqual(get_num_alive(), 1)
        coveralls = None
        self.assertEqual(get_num_alive(), 0)


    def test_errors(self):
        coveralls = Coveralls("test_errors")
        self.assertEqual(coveralls.get_name(), "test_errors")

        with self.assertRaises(CoverallError.TooManyHoles):
            coveralls.maybe_throw(True)

        with self.assertRaisesRegex(InternalError, "expected panic: oh no"):
            coveralls.panic("expected panic: oh no")

if __name__=='__main__':
    unittest.main()
