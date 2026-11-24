from test_package.uniffi_bindgen_tests import *
import unittest

class TestRecords(unittest.TestCase):
    def test_simple_record(self):
        self.assertEqual(roundtrip_simple_rec(SimpleRec(a=42)), SimpleRec(a=42))

    def test_unit_record(self):
        self.assertEqual(UnitRec(), UnitRec())

    def test_record_with_unit_field(self):
        rec = RecWithUnitField(unit=UnitRec(), a=42)
        self.assertEqual(roundtrip_rec_with_unit_field(rec), rec)

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

    def test_methods(self):
        self.assertEqual(SimpleRec(a=42).roundtrip(), SimpleRec(a=42))

    def test_large_record(self):
        large_rec = create_large_rec()
        self.assertEqual(large_rec.a.field_string, "test10")
        self.assertEqual(large_rec.c.field_rec, SimpleRec(a=30))
        self.assertEqual(roundtrip_large_rec(large_rec), large_rec)
        self.assertEqual(large_rec.roundtrip(), large_rec)
        nested_large_rec = create_nested_large_rec()
        self.assertEqual(nested_large_rec.f, large_rec)
        self.assertEqual(roundtrip_nested_large_rec(nested_large_rec), nested_large_rec)

if __name__ == '__main__':
    unittest.main()
