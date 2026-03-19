from test_package.uniffi_bindgen_tests import *
import unittest

class TestCompoundTypes(unittest.TestCase):
    def test_compounds(self):
        self.assertEqual(roundtrip_option(42), 42)
        self.assertEqual(roundtrip_option(None), None)
        self.assertEqual(roundtrip_vec([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_hash_map({ "a": 1, "b": 2 }), { "a": 1, "b": 2 })
        self.assertEqual(roundtrip_complex_compound([
            { "a": 1, "b": 2 }
        ]), [
            { "a": 1, "b": 2 }
        ])
        self.assertEqual(roundtrip_complex_compound(None), None)

if __name__ == '__main__':
    unittest.main()
