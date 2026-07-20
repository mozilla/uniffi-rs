from test_package.uniffi_bindgen_tests import *
import unittest

class TestCustomTypes(unittest.TestCase):
    def test_custom_type(self):
        self.assertEqual(roundtrip_custom_type1(100), 100)

    def test_custom_type_with_config(self):
        self.assertEqual(roundtrip_custom_type2({ "value": 200 }), { "value": 200 })


    def test_types(self):
        self.assertEqual(CustomType1, int)
        self.assertEqual(CustomType2, dict[str, str])

if __name__ == '__main__':
    unittest.main()
