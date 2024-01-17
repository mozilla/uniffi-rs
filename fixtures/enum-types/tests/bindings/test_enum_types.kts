/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.enum_types.*


assert(AnimalUInt.DOG.value == 3.toUByte())
assert(AnimalUInt.CAT.value == 4.toUByte())

assert(AnimalLargeUInt.DOG.value == 4294967298.toULong())
assert(AnimalLargeUInt.CAT.value == 4294967299.toULong())
