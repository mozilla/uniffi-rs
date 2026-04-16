from test_package.uniffi_bindgen_tests import *
from test_package.uniffi_bindgen_tests_external_types_source import (
    ExternalRec, ExternalEnum, ExternalInterface)
import unittest

class TestExternalTypes(unittest.TestCase):
    def test_external_types(self):
        self.assertEqual(roundtrip_ext_record(ExternalRec(a=42)), ExternalRec(a=42))

        self.assertEqual(roundtrip_ext_enum( ExternalEnum.TWO), ExternalEnum.TWO)

        interface = ExternalInterface(20)
        self.assertEqual(roundtrip_ext_interface(interface).get_value(), 20)

        self.assertEqual(roundtrip_ext_custom_type(100), 100)

if __name__ == '__main__':
    unittest.main()
