/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import enum_types


assert(AnimalUInt.dog.rawValue == 3)
assert(AnimalUInt.cat.rawValue == 4)

assert(AnimalLargeUInt.dog.rawValue == 4294967298)
assert(AnimalLargeUInt.cat.rawValue == 4294967299)
