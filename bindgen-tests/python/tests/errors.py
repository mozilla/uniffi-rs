from test_package.uniffi_bindgen_tests import *
import unittest

class TestErrors(unittest.TestCase):
    def test_rust_error_returns(self):
        with self.assertRaises(TestError.Failure1):
            func_with_error(0)

        with self.assertRaises(TestError.Failure2) as cm:
            func_with_error(1)
        self.assertEqual(cm.exception.data, "DATA")

        with self.assertRaises(TestError.Failure3) as cm:
            func_with_error(50)
        self.assertEqual(cm.exception[0], 50)

        with self.assertRaises(TestFlatError.IoError):
            func_with_flat_error(0)

        # These shouldn't throw
        func_with_error(200)
        func_with_flat_error(1)

if __name__ == '__main__':
    unittest.main()
