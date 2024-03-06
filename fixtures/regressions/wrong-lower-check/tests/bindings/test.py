# Check that we can call the function and get a value back

from regression_test_wrong_lower_check import *
import unittest

class WrongLowerCheck(unittest.TestCase):
    def test_works(self):
        self.assertEqual(optional_string(), None)
        self.assertEqual(optional_string("value"), "value")

    def test_klass_works(self):
        self.assertEqual(Klass().optional_string(), None)
        self.assertEqual(Klass().optional_string("value"), "value")

    def test_raises(self):
        self.assertRaises(TypeError, lambda: optional_string(1))
        self.assertRaises(TypeError, lambda: Klass().optional_string(1))

if __name__ == "__main__":
    unittest.main()
