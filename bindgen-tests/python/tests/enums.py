from test_package.uniffi_bindgen_tests import *
import unittest

class TestEnums(unittest.TestCase):
    def test_simple_enums(self):
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.A(10)),
            EnumWithData.A(10))
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.B("Ten")),
            EnumWithData.B("Ten"))
        self.assertEqual(
            roundtrip_enum_with_data(EnumWithData.C()) ,
            EnumWithData.C())

    def test_complex_enums(self):
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.A(EnumNoData.C)),
            ComplexEnum.A(EnumNoData.C))
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.B(EnumWithData.A(20))),
            ComplexEnum.B(EnumWithData.A(20)))
        self.assertEqual(
            roundtrip_complex_enum(ComplexEnum.C(SimpleRec(a=30))),
            ComplexEnum.C(SimpleRec(a=30)))

    def test_discriminents(self):
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

if __name__ == '__main__':
    unittest.main()
