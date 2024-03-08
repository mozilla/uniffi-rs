# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
from enum_types import *

assert(Animal.DOG.value == 0)
assert(Animal.CAT.value == 1)

assert(AnimalNoReprInt.DOG.value == 3)
assert(AnimalNoReprInt.CAT.value == 4)

assert(AnimalUInt.DOG.value == 3)
assert(AnimalUInt.CAT.value == 4)

assert(AnimalLargeUInt.DOG.value == (4294967295 + 3))
assert(AnimalLargeUInt.CAT.value == (4294967295 + 4))
