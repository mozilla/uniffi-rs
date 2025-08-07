/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import enum_types


assert(AnimalUInt.dog.rawValue == 3)
assert(AnimalUInt.cat.rawValue == 4)

assert(AnimalLargeUInt.dog.rawValue == 4294967298)
assert(AnimalLargeUInt.cat.rawValue == 4294967299)

assert(AnimalSignedInt.dog.rawValue == -3)

do {
    let ae = getAnimalEnum(animal: Animal.dog)
    // Can't compare these enums for equality - #2409.
    // assert(ae == ae)

    switch ae {
        case .dog(let o):
            assert(o.getRecord().name == "dog")
        default:
            assert(false)
    }

    switch getAnimalEnum(animal: Animal.cat) {
        case .cat(let r):
            assert(r.name == "cat")
        default:
            assert(false)
    }
}

switch NamedEnumWithDefaults.i() {
    case .i(let d, let e):
        assert(d == 0)
        assert(e == 1)
}

switch NamedEnumWithDefaults.i(d: 2) {
    case .i(let d, let e):
        assert(d == 2)
        assert(e == 1)
}
