import unittest
from test_package.uniffi_bindgen_tests import *

class TestRustTraits(unittest.TestCase):
    def test_debug(self):
        trait_test = RustTraitTest(a=1, b=2)
        self.assertEqual(repr(trait_test), "debug-test-string")

    def test_display(self):
        trait_test = RustTraitTest(a=1, b=2)
        self.assertEqual(str(trait_test), "display-test-string")

    def test_eq(self):
        # The Rust code only uses `a` for the equality
        self.assertEqual(RustTraitTest(a=1, b=2), RustTraitTest(a=1, b=3))
        self.assertNotEqual(RustTraitTest(a=2, b=2), RustTraitTest(a=1, b=2))

    def test_ord(self):
        # The Rust code only uses `a` for the ordering
        self.assertTrue(RustTraitTest(a=1, b=2) < RustTraitTest(a=2, b=3))
        self.assertTrue(
            (RustTraitTest(a=1, b=2) <= RustTraitTest(a=1, b=3))
            and (RustTraitTest(a=1, b=2) >= RustTraitTest(a=1, b=3))
        )

    def test_hash(self):
        # The Rust code only uses `a` for the hash
        self.assertEqual(hash(RustTraitTest(a=1, b=2)), hash(RustTraitTest(a=1, b=3)))
        self.assertNotEqual(hash(RustTraitTest(a=2, b=2)), hash(RustTraitTest(a=1, b=2)))

if __name__ == '__main__':
    unittest.main()

