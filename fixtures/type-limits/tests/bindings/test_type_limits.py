# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_type_limits import *

import unittest

class TestTypeLimits(unittest.TestCase):
    def test_strict_lower_bounds(self):
        self.assertRaises(ValueError, lambda: take_i8(-2**7 - 1))
        self.assertRaises(ValueError, lambda: take_i16(-2**15 - 1))
        self.assertRaises(ValueError, lambda: take_i32(-2**31 - 1))
        self.assertRaises(ValueError, lambda: take_i64(-2**63 - 1))
        self.assertRaises(ValueError, lambda: take_u8(-1))
        self.assertRaises(ValueError, lambda: take_u16(-1))
        self.assertRaises(ValueError, lambda: take_u32(-1))
        self.assertRaises(ValueError, lambda: take_u64(-1))

        self.assertEqual(take_i8(-2**7), -2**7)
        self.assertEqual(take_i16(-2**15), -2**15)
        self.assertEqual(take_i32(-2**31), -2**31)
        self.assertEqual(take_i64(-2**63), -2**63)
        self.assertEqual(take_u8(0), 0)
        self.assertEqual(take_u16(0), 0)
        self.assertEqual(take_u32(0), 0)
        self.assertEqual(take_u64(0), 0)

    def test_strict_upper_bounds(self):
        self.assertRaises(ValueError, lambda: take_i8(2**7))
        self.assertRaises(ValueError, lambda: take_i16(2**15))
        self.assertRaises(ValueError, lambda: take_i32(2**31))
        self.assertRaises(ValueError, lambda: take_i64(2**63))
        self.assertRaises(ValueError, lambda: take_u8(2**8))
        self.assertRaises(ValueError, lambda: take_u16(2**16))
        self.assertRaises(ValueError, lambda: take_u32(2**32))
        self.assertRaises(ValueError, lambda: take_u64(2**64))

        self.assertEqual(take_i8(2**7 - 1), 2**7 - 1)
        self.assertEqual(take_i16(2**15 - 1), 2**15 - 1)
        self.assertEqual(take_i32(2**31 - 1), 2**31 - 1)
        self.assertEqual(take_i64(2**63 - 1), 2**63 - 1)
        self.assertEqual(take_u8(2**8 - 1), 2**8 - 1)
        self.assertEqual(take_u16(2**16 - 1), 2**16 - 1)
        self.assertEqual(take_u32(2**32 - 1), 2**32 - 1)
        self.assertEqual(take_u64(2**64 - 1), 2**64 - 1)

    def test_larger_numbers(self):
        self.assertRaises(ValueError, lambda: take_i8(10**3))
        self.assertRaises(ValueError, lambda: take_i16(10**5))
        self.assertRaises(ValueError, lambda: take_i32(10**10))
        self.assertRaises(ValueError, lambda: take_i64(10**19))
        self.assertRaises(ValueError, lambda: take_u8(10**3))
        self.assertRaises(ValueError, lambda: take_u16(10**5))
        self.assertRaises(ValueError, lambda: take_u32(10**10))
        self.assertRaises(ValueError, lambda: take_u64(10**20))

        self.assertEqual(take_i8(10**2), 10**2)
        self.assertEqual(take_i16(10**4), 10**4)
        self.assertEqual(take_i32(10**9), 10**9)
        self.assertEqual(take_i64(10**18), 10**18)
        self.assertEqual(take_u8(10**2), 10**2)
        self.assertEqual(take_u16(10**4), 10**4)
        self.assertEqual(take_u32(10**9), 10**9)
        self.assertEqual(take_u64(10**19), 10**19)

if __name__ == "__main__":
    unittest.main()
