from test_package.uniffi_bindgen_tests import *
import unittest

class TestEnums(unittest.TestCase):
    def test_simple_enums(self):
        self.assertEqual(roundtrip_enum_no_data(EnumNoData.A), EnumNoData.A)
        self.assertEqual(roundtrip_enum_no_data(EnumNoData.B), EnumNoData.B)
        self.assertEqual(roundtrip_enum_no_data(EnumNoData.C), EnumNoData.C)
        # passing in an int value rather than an enum should fail
        with self.assertRaises(ValueError):
            roundtrip_enum_no_data(1)

    def test_enums_with_data(self):
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.A(10, 20)),
            EnumWithData.A(10, 20))
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.B("Ten", 10)),
            EnumWithData.B("Ten", 10))
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.C()) ,
            EnumWithData.C())
        # passing in an int value rather than an enum should fail
        with self.assertRaises(ValueError):
            roundtrip_enum_with_data(1)

    def test_enum_fields(self):
        a = EnumWithData.A(10, 20)
        self.assertEqual(a.value, 10)
        self.assertEqual(a.value2, 20)

        b = EnumWithData.B("Ten", 10)
        self.assertEqual(b[0], "Ten")
        self.assertEqual(b[1], 10)

    def test_complex_enums(self):
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.A(EnumNoData.C)),
            ComplexEnum.A(EnumNoData.C))
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.B(EnumWithData.A(20, 40))),
            ComplexEnum.B(EnumWithData.A(20, 40)))
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.C(SimpleRec(a=30))),
            ComplexEnum.C(SimpleRec(a=30)))

    def test_discriminants(self):
        # Test that the enum discriminant values

        # All discriminants specified, use the specified values
        self.assertEqual(ExplicitValuedEnum.FIRST.value, 1)
        self.assertEqual(ExplicitValuedEnum.SECOND.value, 2)
        self.assertEqual(ExplicitValuedEnum.FOURTH.value, 4)
        self.assertEqual(ExplicitValuedEnum.TENTH.value, 10)
        self.assertEqual(ExplicitValuedEnum.ELEVENTH.value, 11)
        self.assertEqual(ExplicitValuedEnum.THIRTEENTH.value, 13)

        # Some discriminants specified, increment by one for any unspecified variants
        self.assertEqual(GappedEnum.ONE.value, 10)
        self.assertEqual(GappedEnum.TWO.value, 11) # Sequential value after ONE (10+1)
        self.assertEqual(GappedEnum.THREE.value, 14) # Explicit value again

    def test_large_discriminants(self):
        self.assertEqual(EnumWithLargeDiscriminants.ONE.value, 4294967295 + 3)
        self.assertEqual(EnumWithLargeDiscriminants.TWO.value, 4294967295 + 4)
        self.assertEqual(
            roundtrip_enum_with_large_discriminants(EnumWithLargeDiscriminants.ONE),
            EnumWithLargeDiscriminants.ONE
        )

    def test_methods(self):
        self.assertEqual(EnumWithData.A(1, 0).roundtrip(), EnumWithData.A(1, 0))

if __name__ == '__main__':
    unittest.main()
