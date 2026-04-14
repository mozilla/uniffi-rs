from test_package.uniffi_bindgen_tests import *
import unittest
from datetime import datetime, timedelta, timezone

class TestTime(unittest.TestCase):
    def test_bytes(self):
        self.assertEqual(roundtrip_bytes(b'test-data'), b'test-data')

if __name__ == '__main__':
    unittest.main()

