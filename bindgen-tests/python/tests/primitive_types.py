from test_package.uniffi_bindgen_tests import *
import unittest

class TestPrimitiveTypes(unittest.TestCase):
    def test_input(self):
        input_u8(42)
        input_i8(-42)
        input_u16(42)
        input_i16(-42)
        input_u32(42)
        input_i32(-42)
        input_u64(42)
        input_i64(-42)
        input_f32(0.5)
        input_f64(-3.5)
        input_bool(True)
        input_string("ABC")

    def test_output(self):
        self.assertEqual(output_u8(), 1)
        self.assertEqual(output_i8(), 1)
        self.assertEqual(output_u16(), 1)
        self.assertEqual(output_i16(), 1)
        self.assertEqual(output_u32(), 1)
        self.assertEqual(output_i32(), 1)
        self.assertEqual(output_u64(), 1)
        self.assertEqual(output_i64(), 1)
        self.assertEqual(output_f32(), 1.0)
        self.assertEqual(output_f64(), 1.0)
        self.assertEqual(output_bool(), True)
        self.assertEqual(output_string(), "test-string")

    def test_roundtrip(self):
        self.assertEqual(roundtrip_u8(42), 42)
        self.assertEqual(roundtrip_i8(-42), -42)
        self.assertEqual(roundtrip_u16(42), 42)
        self.assertEqual(roundtrip_i16(-42), -42)
        self.assertEqual(roundtrip_u32(42), 42)
        self.assertEqual(roundtrip_i32(-42), -42)
        self.assertEqual(roundtrip_u64(42), 42)
        self.assertEqual(roundtrip_i64(-42), -42)
        self.assertEqual(roundtrip_f32(0.5), 0.5)
        self.assertEqual(roundtrip_f64(-3.5), -3.5)
        self.assertEqual(roundtrip_bool(True), True)
        self.assertEqual(roundtrip_string("ABC"), "ABC")
        # Test calling a function with lots of args
        # This function will sum up all the numbers, then negate the value since we passed in `true`
        self.assertEqual(sum_with_many_types(1, -2, 3, -4, 5, -6, 7, -8, 9.5, -10.5, True), 5)

if __name__ == '__main__':
    unittest.main()
