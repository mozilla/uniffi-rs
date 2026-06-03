from test_package.uniffi_bindgen_tests import *
import unittest

class TestOptions(unittest.TestCase):
    def test_options(self):
        self.assertEqual(roundtrip_option_u8(67), 67)
        self.assertEqual(roundtrip_option_u8(None), None)
        self.assertEqual(roundtrip_option_i8(67), 67)
        self.assertEqual(roundtrip_option_i8(None), None)
        self.assertEqual(roundtrip_option_u16(67), 67)
        self.assertEqual(roundtrip_option_u16(None), None)
        self.assertEqual(roundtrip_option_i16(67), 67)
        self.assertEqual(roundtrip_option_i16(None), None)
        self.assertEqual(roundtrip_option_u32(67), 67)
        self.assertEqual(roundtrip_option_u32(None), None)
        self.assertEqual(roundtrip_option_i32(67), 67)
        self.assertEqual(roundtrip_option_i32(None), None)
        self.assertEqual(roundtrip_option_u64(67), 67)
        self.assertEqual(roundtrip_option_u64(None), None)
        self.assertEqual(roundtrip_option_i64(67), 67)
        self.assertEqual(roundtrip_option_i64(None), None)
        self.assertEqual(roundtrip_option_string("test-string"), "test-string")
        self.assertEqual(roundtrip_option_string(None), None)
        self.assertEqual(roundtrip_option_bool(True), True)
        self.assertEqual(roundtrip_option_bool(None), None)
        self.assertEqual(roundtrip_option_rec(OptionsRec(a=67)), OptionsRec(a=67))
        self.assertEqual(roundtrip_option_rec(None), None)

if __name__ == '__main__':
    unittest.main()
