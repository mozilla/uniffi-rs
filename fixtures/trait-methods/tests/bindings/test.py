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

class TestTraitRecord(unittest.TestCase):
    def test_repr(self):
        r = TraitRecord(s="yo", i=2)
        self.assertEqual(repr(r), 'TraitRecord { s: "yo", i: 2 }')

    def test_eq(self):
        self.assertEqual(TraitRecord(s="yo", i=2), TraitRecord(s="yo", i=2))
        self.assertEqual(TraitRecord(s="yo", i=2), TraitRecord(s="yo", i=3))
        self.assertNotEqual(TraitRecord(s="yo", i=2), TraitRecord(s="hi", i=2))

    def test_hash(self):
        d = {}
        r1 = TraitRecord(s="yo", i=2)
        r2 = TraitRecord(s="yo", i=3)
        d[r1] = "m"
        self.assertTrue(r2 in d)

    def test_ord(self):
        r1 = TraitRecord(s="a", i=0)
        r2 = TraitRecord(s="b", i=0)
        self.assertLess(r1, r2)

class TestUdlRecord(unittest.TestCase):
    def test_repr(self):
        r = UdlRecord(s="yo", i=2)
        self.assertEqual(repr(r), 'UdlRecord { s: "yo", i: 2 }')

    def test_eq(self):
        self.assertEqual(UdlRecord(s="yo", i=2), UdlRecord(s="yo", i=2))
        self.assertEqual(UdlRecord(s="yo", i=2), UdlRecord(s="yo", i=3))
        self.assertNotEqual(UdlRecord(s="yo", i=2), UdlRecord(s="hi", i=3))

    def test_hash(self):
        d = {}
        r1 = UdlRecord(s="yo", i=2)
        r2 = UdlRecord(s="yo", i=3)
        d[r1] = "m"
        self.assertTrue(r2 in d)

    def test_ord(self):
        self.assertLess(UdlRecord(s="a", i=0), UdlRecord(s="b", i=0))

class TestTraitEnum(unittest.TestCase):
    def test_str(self):
        m = TraitEnum.S("yo")
        self.assertEqual(str(m), 'TraitEnum::S("yo")')

    def test_repr(self):
        m = TraitEnum.S("yo")
        self.assertEqual(repr(m), 'S("yo")')

    def test_eq(self):
        self.assertEqual(TraitEnum.S("1"), TraitEnum.S("1"))
        self.assertEqual(TraitEnum.S("1"), TraitEnum.S("2"))
        self.assertEqual(TraitEnum.I(1), TraitEnum.I(1))
        self.assertEqual(TraitEnum.I(1), TraitEnum.I(2))
        self.assertNotEqual(TraitEnum.S("1"), TraitEnum.I(1))

    def test_eq_wrong_type(self):
        self.assertNotEqual(TraitEnum.S("1"), 17)

    def test_hash(self):
        d = {}
        m = TraitEnum.S("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        s1 = TraitEnum.S("1")
        i1 = TraitEnum.I(1)
        self.assertLess(s1, i1)
        self.assertLessEqual(s1, i1)

class TestUdlEnum(unittest.TestCase):
    def test_repr(self):
        m = UdlEnum.S("yo")
        self.assertEqual(repr(m), 'S { s: "yo" }')

    def test_eq(self):
        self.assertEqual(UdlEnum.S("1"), UdlEnum.S("1"))
        self.assertEqual(UdlEnum.S("1"), UdlEnum.S("2"))
        self.assertEqual(UdlEnum.I(1), UdlEnum.I(1))
        self.assertEqual(UdlEnum.I(1), UdlEnum.I(2))
        self.assertNotEqual(UdlEnum.S("1"), UdlEnum.I(1))

    def test_eq_wrong_type(self):
        self.assertNotEqual(UdlEnum.S("1"), 17)

    def test_hash(self):
        d = {}
        m = UdlEnum.S("m")
        d[m] = "m"
        self.assertTrue(m in d)

    def test_ord(self):
        self.assertLess(UdlEnum.S("1"), UdlEnum.I(1))
        self.assertLessEqual(UdlEnum.S("1"), UdlEnum.I(2))

class TestOddNamed(unittest.TestCase):
    def test_odd_names(self):
        self.assertEqual(XyzEnum.XYZ_NONE(), XyzEnum.XYZ_NONE())

if __name__=='__main__':
    unittest.main()
