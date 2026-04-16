from test_package.uniffi_bindgen_tests import *
import unittest

class TestSimpleFns(unittest.TestCase):
    def test_simple_function(self):
        # the test here is just that we can successfully call a function across the FFI
        test_func()

    def test_unexpected_error(self):
        with self.assertRaises(InternalError):
            test_unexpected_error_func()

if __name__ == '__main__':
    unittest.main()
