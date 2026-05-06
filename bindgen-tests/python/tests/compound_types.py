from test_package.uniffi_bindgen_tests import *
import unittest

class TestCompoundTypes(unittest.TestCase):
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
        self.assertEqual(roundtrip_option_rec(CompoundTypesRec(a=67)), CompoundTypesRec(a=67))
        self.assertEqual(roundtrip_option_rec(None), None)

    def test_vecs(self):
        self.assertEqual(roundtrip_vec_i8([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_u16([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_i16([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_u32([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_i32([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_u64([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_i64([1, 2, 3]), [1, 2, 3])
        self.assertEqual(roundtrip_vec_string(["test-string"]), ["test-string"])
        self.assertEqual(roundtrip_vec_bool([True, False]), [True, False])
        self.assertEqual(roundtrip_vec_rec([CompoundTypesRec(a=67)]), [CompoundTypesRec(a=67)])

    def test_hash_maps(self):
        self.assertEqual(roundtrip_hash_map({ "a": 1, "b": 2 }), { "a": 1, "b": 2 })
        self.assertEqual(roundtrip_hash_set({"a", "b", "c"}), {"a", "b", "c"})
        self.assertEqual(roundtrip_hash_map_u32_key({ 1: 2, 2: 4 }), { 1: 2, 2: 4})

    def test_rec_with_compounds(self):
        self.assertEqual(
            RecWithCompounds(
                a=EnumWithCompounds.A(None),
                b=None,
                c=[True, False],
                d={
                    "a": 10,
                    "b": 20,
                },
            ),
            RecWithCompounds(
                a=EnumWithCompounds.A(None),
                b=None,
                c=[True, False],
                d={
                    "a": 10,
                    "b": 20,
                },
            ),
        )

    def test_complex(self):
        self.assertEqual(roundtrip_complex_compound([
            {
                "a": CompoundTypesComplexRec(a=10, b="Test", c=CompoundTypesEnum.A(100)),
                "b": CompoundTypesComplexRec(a=20, b="Test2", c=CompoundTypesEnum.B(a=1.0, b=True)),
            }
        ]), [
            {
                "a": CompoundTypesComplexRec(a=10, b="Test", c=CompoundTypesEnum.A(100)),
                "b": CompoundTypesComplexRec(a=20, b="Test2", c=CompoundTypesEnum.B(a=1.0, b=True)),
            }
        ])
        self.assertEqual(roundtrip_complex_compound(None), None)
        self.assertEqual(roundtrip_complex_hash_set([{"a", "b"}]), [{"a", "b"}])
        self.assertEqual(roundtrip_complex_hash_set(None), None)

if __name__ == '__main__':
    unittest.main()
