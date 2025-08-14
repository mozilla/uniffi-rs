/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.enum_types.*
import kotlin.reflect.full.functions

assert(AnimalUInt.DOG.value == 3.toUByte())
assert(AnimalUInt.CAT.value == 4.toUByte())

assert(AnimalLargeUInt.DOG.value == 4294967298.toULong())
assert(AnimalLargeUInt.CAT.value == 4294967299.toULong())

// could check `value == (-3).toByte()` but that's ugly :)
assert(AnimalSignedInt.DOG.value + 3 == 0)

// destroy function semantics
// Assert that no destroy() function is created for simple Enum
val simpleCat: Animal = Animal.CAT
assert(simpleCat::class.functions.find { it.name == "destroy" } == null)

// Assert that destroy() function is created for Enum with variants containing an object
// Even though we are creating a non-object variant we still get it.
val n: AnimalEnum = AnimalEnum.None
assert(n::class.functions.find { it.name == "destroy" } != null)

getAnimalEnum(Animal.DOG).let { a ->
    assert(a is AnimalEnum.Dog)
    assert(a == getAnimalEnum(Animal.DOG))
    // markh can't work out how to make this work!?
    // assert(a.v1.getRecord().name == "dog")
}

assert(NamedEnumWithDefaults.I().d == 0U.toUByte())
assert(NamedEnumWithDefaults.I().e == 1U.toUByte())
assert(NamedEnumWithDefaults.I(e=2U).d == 0U.toUByte())
assert(NamedEnumWithDefaults.I(e=2U).e == 2U.toUByte())