# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
from enum_types import *

class TestErrorTypes(unittest.TestCase):
    def test_animals(self):
        self.assertEqual(Animal.DOG.value,  0)
        self.assertEqual(Animal.CAT.value, 1)
        self.assertEqual(get_animal(None), Animal.DOG)
        with self.assertRaises(ValueError):
            get_animal(1)

        self.assertEqual(AnimalNoReprInt.DOG.value, 3)
        self.assertEqual(AnimalNoReprInt.CAT.value, 4)

        self.assertEqual(AnimalUInt.DOG.value, 3)
        self.assertEqual(AnimalUInt.CAT.value, 4)

        self.assertEqual(AnimalLargeUInt.DOG.value, 4294967295 + 3)
        self.assertEqual(AnimalLargeUInt.CAT.value, 4294967295 + 4)

        self.assertEqual(AnimalSignedInt.DOG.value, -3)
        self.assertEqual(AnimalSignedInt.CAT.value, -2)
        self.assertEqual(AnimalSignedInt.KOALA.value, -1)
        self.assertEqual(AnimalSignedInt.WALLABY.value, 0)
        self.assertEqual(AnimalSignedInt.WOMBAT.value, 1)

    def test_containers(self):
        self.assertTrue(get_animal_enum(Animal.DOG).is_DOG())
        self.assertEqual(get_animal_enum(Animal.DOG)[0].get_record().name, "dog")
        self.assertEqual(get_animal_enum(Animal.DOG), get_animal_enum(Animal.DOG))
        self.assertEqual(get_animal_enum(Animal.CAT)[0].name, "cat")
        self.assertEqual(get_animal_enum(Animal.CAT), get_animal_enum(Animal.CAT))
        self.assertNotEqual(get_animal_enum(Animal.DOG), get_animal_enum(Animal.CAT))

    def test_defaults(self):
        e = NamedEnumWithDefaults.I()
        self.assertEqual(e.d, 0)
        self.assertEqual(e.e, 1)
        e = NamedEnumWithDefaults.I(e=2)
        self.assertEqual(e.d, 0)
        self.assertEqual(e.e, 2)

if __name__=='__main__':
    unittest.main()
