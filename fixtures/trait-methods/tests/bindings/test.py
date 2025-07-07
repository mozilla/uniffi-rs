# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import unittest
from trait_methods import *

class TestTraitMethods(unittest.TestCase):
    def test_str(self):
        m = TraitMethods("yo")
        self.assertEqual(str(m), "TraitMethods(yo)")

    def test_repr(self):
        m = TraitMethods("yo")
        self.assertEqual(repr(m), 'TraitMethods { val: "yo" }')

    def test_eq(self):
        m = TraitMethods("yo")
        self.assertEqual(m, TraitMethods("yo"))
        self.assertNotEqual(m, TraitMethods("yoyo"))

    def test_eq_wrong_type(self):
        m = TraitMethods("yo")
        self.assertNotEqual(m, 17)

    def test_hash(self):
        d = {}
        m = TraitMethods("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        a = TraitMethods("a")
        b = TraitMethods("b")
        self.assertLess(a, b)
        self.assertLessEqual(a, b)
        self.assertLessEqual(a, a)
        self.assertGreater(b, a)
        self.assertGreaterEqual(b, a)
        self.assertGreaterEqual(b, b)

class TestProcmacroTraitMethods(unittest.TestCase):
    def test_str(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(str(m), "ProcTraitMethods(yo)")

    def test_repr(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(repr(m), 'ProcTraitMethods { val: "yo" }')

    def test_eq(self):
        m = ProcTraitMethods("yo")
        self.assertEqual(m, ProcTraitMethods("yo"))
        self.assertNotEqual(m, ProcTraitMethods("yoyo"))

    def test_eq(self):
        m = ProcTraitMethods("yo")
        self.assertNotEqual(m, 17)

    def test_hash(self):
        d = {}
        m = ProcTraitMethods("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        a = ProcTraitMethods("a")
        b = ProcTraitMethods("b")
        self.assertLess(a, b)
        self.assertLessEqual(a, b)
        self.assertLessEqual(a, a)
        self.assertGreater(b, a)
        self.assertGreaterEqual(b, a)
        self.assertGreaterEqual(b, b)

if __name__=='__main__':
    unittest.main()
