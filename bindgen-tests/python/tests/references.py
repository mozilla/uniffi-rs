from test_package.uniffi_bindgen_tests import *
import unittest

class TestReferences(unittest.TestCase):
    def test_value_ref(self):
        self.assertEqual(roundtrip_u8_ref(2), 2)

    def test_interface_ref(self):
        interface = ReferenceTestInterface()
        self.assertEqual(interface.double_value(2), 4)
        self.assertEqual(call_double_value(interface, 3), 6)

    def test_trait_interface_ref(self):
        trait_interface = create_reference_test_trait_interface()
        self.assertEqual(call_triple_value_trait_interface(trait_interface, 10), 30)

if __name__ == '__main__':
    unittest.main()

