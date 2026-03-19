from test_package.uniffi_bindgen_tests import *
import unittest

class TestComplexFns(unittest.TestCase):
    def test_default_arguments(self):
        self.assertEqual(func_with_default(), "DEFAULT")
        self.assertEqual(func_with_default("NON-DEFAULT"), "NON-DEFAULT")

        complex_methods = ComplexMethods()
        self.assertEqual(complex_methods.method_with_default(), "DEFAULT")
        self.assertEqual(complex_methods.method_with_default("NON-DEFAULT"), "NON-DEFAULT")

    def test_argument_name_mapping(self):
        complex_methods = ComplexMethods()
        # the following calls will fail if the argument name differs
        func_with_multi_word_arg(the_argument="test")
        complex_methods.method_with_multi_word_arg(the_argument="test")

if __name__ == '__main__':
    unittest.main()
