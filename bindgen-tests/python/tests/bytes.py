from test_package.uniffi_bindgen_tests import *
import unittest
from datetime import datetime, timedelta, timezone

class TestTime(unittest.TestCase):
    def test_bytes(self):
        self.assertEqual(roundtrip_bytes(b'test-data'), b'test-data')

    def test_zero_copy_bytes(self):
        # Zero-copy &[u8]. Accepts both bytes and bytearray.
        self.assertEqual(sum_bytes(b""), 0)
        self.assertEqual(sum_bytes(b"\x01\x02\x03"), 6)
        self.assertEqual(sum_bytes(bytearray([1, 2, 3])), 6)
        self.assertEqual(sum_bytes(bytearray(0)), 0)
        self.assertIsNone(first_byte(b""))
        self.assertEqual(first_byte(b"\x2a"), 42)
        self.assertEqual(first_byte(bytearray([42])), 42)

    def test_zero_copy_bytes_mut(self):
        # Zero-copy &mut [u8]. Rust writes land in place.
        buf = bytearray(4)
        fill_bytes(buf)
        self.assertEqual(buf, bytearray([0, 1, 2, 3]))

        inc = bytearray([1, 2, 3])
        increment_bytes(inc)
        self.assertEqual(inc, bytearray([2, 3, 4]))

        # Empty buffer is handled without crashing.
        empty = bytearray(0)
        fill_bytes(empty)
        self.assertEqual(empty, bytearray(0))

if __name__ == '__main__':
    unittest.main()

