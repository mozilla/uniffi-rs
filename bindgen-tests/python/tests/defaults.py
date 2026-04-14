from test_package.uniffi_bindgen_tests import *
import unittest

class TestDefaults(unittest.TestCase):
    def test_records(self):
        r = RecWithDefault()
        self.assertEqual(r.n, 42)
        self.assertEqual(r.v, [])

    def test_enums(self):
        e = EnumWithDefault.OTHER_VARIANT()
        self.assertEqual(e.a, "default")

    def test_arguments(self):
        self.assertEqual(func_with_default(), "DEFAULT")
        self.assertEqual(func_with_default("NON-DEFAULT"), "NON-DEFAULT")

        i = InterfaceWithDefaults()
        self.assertEqual(i.method_with_default(), "DEFAULT")
        self.assertEqual(i.method_with_default("NON-DEFAULT"), "NON-DEFAULT")

if __name__ == '__main__':
    unittest.main()
