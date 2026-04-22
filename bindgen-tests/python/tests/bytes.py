from test_package.uniffi_bindgen_tests import *
import unittest
from datetime import datetime, timedelta, timezone

class TestTime(unittest.TestCase):
    def test_bytes(self):
        self.assertEqual(roundtrip_bytes(b'test-data'), b'test-data')

    def test_zero_copy_bytes(self):
        # Zero-copy &[u8] — proc-macro path
        self.assertEqual(sum_bytes_procmacro(b""), 0)
        self.assertEqual(sum_bytes_procmacro(b"\x01\x02\x03"), 6)
        self.assertIsNone(first_byte_procmacro(b""))
        self.assertEqual(first_byte_procmacro(b"\x2a"), 42)

if __name__ == '__main__':
    unittest.main()

