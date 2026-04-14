from test_package.uniffi_bindgen_tests import *
import unittest
from datetime import datetime, timedelta, timezone

class TestTime(unittest.TestCase):
    def test_duration(self):
        self.assertEqual(
            roundtrip_duration(timedelta(days=1, seconds=2)),
            timedelta(days=1, seconds=2),
        )

    def test_systemtime(self):
        self.assertEqual(
            roundtrip_systemtime(datetime(2000, 1, 1, tzinfo=timezone.utc)),
            datetime(2000, 1, 1, tzinfo=timezone.utc)
        )

if __name__ == '__main__':
    unittest.main()

