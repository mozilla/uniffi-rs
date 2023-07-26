# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from uniffi_type_limits import *

import math
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

    def test_non_integer(self):
        self.assertRaises(TypeError, lambda: take_i8(None))
        self.assertRaises(TypeError, lambda: take_i16(None))
        self.assertRaises(TypeError, lambda: take_i32(None))
        self.assertRaises(TypeError, lambda: take_i64(None))
        self.assertRaises(TypeError, lambda: take_u8(None))
        self.assertRaises(TypeError, lambda: take_u16(None))
        self.assertRaises(TypeError, lambda: take_u32(None))
        self.assertRaises(TypeError, lambda: take_u64(None))

        self.assertRaises(TypeError, lambda: take_i8("0"))
        self.assertRaises(TypeError, lambda: take_i16("0"))
        self.assertRaises(TypeError, lambda: take_i32("0"))
        self.assertRaises(TypeError, lambda: take_i64("0"))
        self.assertRaises(TypeError, lambda: take_u8("0"))
        self.assertRaises(TypeError, lambda: take_u16("0"))
        self.assertRaises(TypeError, lambda: take_u32("0"))
        self.assertRaises(TypeError, lambda: take_u64("0"))

        self.assertRaises(TypeError, lambda: take_i8(0.0))
        self.assertRaises(TypeError, lambda: take_i16(0.0))
        self.assertRaises(TypeError, lambda: take_i32(0.0))
        self.assertRaises(TypeError, lambda: take_i64(0.0))
        self.assertRaises(TypeError, lambda: take_u8(0.0))
        self.assertRaises(TypeError, lambda: take_u16(0.0))
        self.assertRaises(TypeError, lambda: take_u32(0.0))
        self.assertRaises(TypeError, lambda: take_u64(0.0))

        class A:
            pass

        self.assertRaises(TypeError, lambda: take_i8(A()))
        self.assertRaises(TypeError, lambda: take_i16(A()))
        self.assertRaises(TypeError, lambda: take_i32(A()))
        self.assertRaises(TypeError, lambda: take_i64(A()))
        self.assertRaises(TypeError, lambda: take_u8(A()))
        self.assertRaises(TypeError, lambda: take_u16(A()))
        self.assertRaises(TypeError, lambda: take_u32(A()))
        self.assertRaises(TypeError, lambda: take_u64(A()))

    def test_integer_like(self):
        self.assertEqual(take_i8(123), 123.0)
        self.assertEqual(take_i16(123), 123.0)
        self.assertEqual(take_i32(123), 123.0)
        self.assertEqual(take_i64(123), 123.0)
        self.assertEqual(take_u8(123), 123.0)
        self.assertEqual(take_u16(123), 123.0)
        self.assertEqual(take_u32(123), 123.0)
        self.assertEqual(take_u64(123), 123.0)

        self.assertEqual(take_i8(False), 0)
        self.assertEqual(take_i16(False), 0)
        self.assertEqual(take_i32(False), 0)
        self.assertEqual(take_i64(False), 0)
        self.assertEqual(take_u8(False), 0)
        self.assertEqual(take_u16(False), 0)
        self.assertEqual(take_u32(False), 0)
        self.assertEqual(take_u64(False), 0)

        self.assertEqual(take_i8(True), 1)
        self.assertEqual(take_i16(True), 1)
        self.assertEqual(take_i32(True), 1)
        self.assertEqual(take_i64(True), 1)
        self.assertEqual(take_u8(True), 1)
        self.assertEqual(take_u16(True), 1)
        self.assertEqual(take_u32(True), 1)
        self.assertEqual(take_u64(True), 1)

        class A:
            def __index__(self):
                return 123

        self.assertEqual(take_i8(A()), 123)
        self.assertEqual(take_i16(A()), 123)
        self.assertEqual(take_i32(A()), 123)
        self.assertEqual(take_i64(A()), 123)
        self.assertEqual(take_u8(A()), 123)
        self.assertEqual(take_u16(A()), 123)
        self.assertEqual(take_u32(A()), 123)
        self.assertEqual(take_u64(A()), 123)

    def test_non_float(self):
        self.assertRaises(TypeError, lambda: take_f32(None))
        self.assertRaises(TypeError, lambda: take_f64(None))

        self.assertRaises(TypeError, lambda: take_f32("0"))
        self.assertRaises(TypeError, lambda: take_f64("0"))

        self.assertRaises(TypeError, lambda: take_f32(1j))
        self.assertRaises(TypeError, lambda: take_f64(1j))

        class A:
            pass

        self.assertRaises(TypeError, lambda: take_f32(A()))
        self.assertRaises(TypeError, lambda: take_f64(A()))

    def test_float_like(self):
        self.assertEqual(take_f32(False), 0.0)
        self.assertEqual(take_f64(False), 0.0)

        self.assertEqual(take_f32(True), 1.0)
        self.assertEqual(take_f64(True), 1.0)

        self.assertEqual(take_f32(123), 123.0)
        self.assertEqual(take_f64(123), 123.0)

        class A:
            def __float__(self):
                return 456.0

        self.assertEqual(take_f32(A()), 456.0)
        self.assertEqual(take_f64(A()), 456.0)

    def test_special_floats(self):
        self.assertEqual(take_f32(math.inf), math.inf)
        self.assertEqual(take_f64(math.inf), math.inf)

        self.assertEqual(take_f32(-math.inf), -math.inf)
        self.assertEqual(take_f64(-math.inf), -math.inf)

        self.assertEqual(math.copysign(1.0, take_f32(0.0)), 1.0)
        self.assertEqual(math.copysign(1.0, take_f64(0.0)), 1.0)

        self.assertEqual(math.copysign(1.0, take_f32(-0.0)), -1.0)
        self.assertEqual(math.copysign(1.0, take_f64(-0.0)), -1.0)

        self.assertTrue(math.isnan(take_f32(math.nan)))
        self.assertTrue(math.isnan(take_f64(math.nan)))

    def test_non_string(self):
        self.assertRaises(TypeError, lambda: take_string(None))
        self.assertRaises(TypeError, lambda: take_string(False))
        self.assertRaises(TypeError, lambda: take_string(True))
        self.assertRaises(TypeError, lambda: take_string(0))
        self.assertRaises(TypeError, lambda: take_string(0.0))
        self.assertRaises(TypeError, lambda: take_string(b""))

        class A:
            def __str__(self):
                return ""

        self.assertRaises(TypeError, lambda: take_string(A()))

    def test_strings(self):
        self.assertRaises(ValueError, lambda: take_string("\ud800")) # surrogate
        self.assertEqual(take_string(""), "")
        self.assertEqual(take_string("愛"), "愛")
        self.assertEqual(take_string("💖"), "💖")

    def test_non_bytes(self):
        self.assertRaises(TypeError, lambda: take_bytes(None))
        self.assertRaises(TypeError, lambda: take_bytes(False))
        self.assertRaises(TypeError, lambda: take_bytes(True))
        self.assertRaises(TypeError, lambda: take_bytes(0))
        self.assertRaises(TypeError, lambda: take_bytes(0.0))
        self.assertRaises(TypeError, lambda: take_bytes(""))

        class A:
            def __str__(self):
                return ""

        self.assertRaises(TypeError, lambda: take_bytes(A()))

    def test_bytes(self):
        self.assertEqual(take_bytes(b""), b"")
        self.assertEqual(take_bytes(b"\xff"), b"\xff") # invalid utf-8 byte
        self.assertEqual(take_bytes(b"\xed\xa0\x80"), b"\xed\xa0\x80") # surrogate
        self.assertEqual(take_bytes("愛".encode()), "愛".encode())
        self.assertEqual(take_bytes("💖".encode()), "💖".encode())
        self.assertEqual(take_bytes("愛".encode("utf-16-le")), b"\x1b\x61")
        self.assertEqual(take_bytes("💖".encode("utf-16-le")), b"\x3d\xd8\x96\xdc")

if __name__ == "__main__":
    unittest.main()
