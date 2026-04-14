from test_package.uniffi_bindgen_tests import *
import unittest

class TestRecords(unittest.TestCase):
    def test_simple_record(self):
        self.assertEqual(roundtrip_simple_rec(SimpleRec(a=42)), SimpleRec(a=42))

    def test_unit_record(self):
        self.assertEqual(UnitRec(), UnitRec())

    def test_complex_record(self):
        self.assertEqual(
          roundtrip_complex_rec(
            ComplexRec(
              field_u8=0,
              field_i8=-1,
              field_u16=2,
              field_i16=-3,
              field_u32=4,
              field_i32=-5,
              field_u64=6,
              field_i64=-7,
              field_f32=8.5,
              field_f64=9.5,
              field_string="test",
              field_rec=SimpleRec(a=42)
            )
          ), ComplexRec(
            field_u8=0,
            field_i8=-1,
            field_u16=2,
            field_i16=-3,
            field_u32=4,
            field_i32=-5,
            field_u64=6,
            field_i64=-7,
            field_f32=8.5,
            field_f64=9.5,
            field_string="test",
            field_rec=SimpleRec(a=42)
          )
        )

if __name__ == '__main__':
    unittest.main()
