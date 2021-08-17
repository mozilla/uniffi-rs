import unittest
from json_wrapper_lib import *

class TestIt(unittest.TestCase):
    def test_get(self):
        obj = {
            "test-key": "test-value",
        }
        self.assertEqual(get(obj, "test-key"), "test-value")
        self.assertEqual(get(obj, "missing-key"), None)

if __name__=='__main__':
    unittest.main()
