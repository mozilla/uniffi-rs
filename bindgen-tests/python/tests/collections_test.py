from test_package.uniffi_bindgen_tests import *
import unittest

class TestCollections(unittest.TestCase):
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
        self.assertEqual(roundtrip_vec_rec([CollectionsRec(a=67)]), [CollectionsRec(a=67)])

    def test_hash_maps(self):
        self.assertEqual(roundtrip_hash_map({ "a": 1, "b": 2 }), { "a": 1, "b": 2 })
        self.assertEqual(roundtrip_hash_set({"a", "b", "c"}), {"a", "b", "c"})
        self.assertEqual(roundtrip_hash_map_u32_key({ 1: 2, 2: 4 }), { 1: 2, 2: 4})

    def test_rec_with_collections(self):
        self.assertEqual(
            roundtrip_rec_with_collections(
                RecWithCollections(
                    a=EnumWithCollections.A(None),
                    b=None,
                    c=[True, False],
                    d={
                        "a": 10,
                        "b": 20,
                    },
                ),
            ),
            RecWithCollections(
                a=EnumWithCollections.A(None),
                b=None,
                c=[True, False],
                d={
                    "a": 10,
                    "b": 20,
                },
            ),
        )

    def test_complex(self):
        self.assertEqual(roundtrip_vec_hash_set([{"a", "b"}]), [{"a", "b"}])
        self.assertEqual(roundtrip_vec_hash_set(None), None)
        self.assertEqual(roundtrip_complex_collection_type([
            {
                "a": CollectionsComplexRec(a=10, b="Test", c=CollectionsEnum.A(100)),
                "b": CollectionsComplexRec(a=20, b="Test2", c=CollectionsEnum.B(a=1.0, b=True)),
            }
        ]), [
            {
                "a": CollectionsComplexRec(a=10, b="Test", c=CollectionsEnum.A(100)),
                "b": CollectionsComplexRec(a=20, b="Test2", c=CollectionsEnum.B(a=1.0, b=True)),
            }
        ])
        self.assertEqual(roundtrip_complex_collection_type(None), None)

if __name__ == '__main__':
    unittest.main()
