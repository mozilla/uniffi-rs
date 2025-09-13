/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_fixture_rename.*;

// Test renamed types - should be accessible by their custom names
val record = RenamedRecord(item = 42)
assert(record.item == 42)

val enum1 = RenamedEnum.VariantA
val enum2 = RenamedEnum.Record(record)

// Test renamed function
val result = renamedFunction(record)
assert(result is RenamedEnum.Record)
assert((result as RenamedEnum.Record).v1.item == 42)

// Test renamed object with renamed constructor and method
val obj = RenamedObject.renamedConstructor(123)
assert(obj is RenamedObject)
val methodResult = obj.renamedMethod()
assert(methodResult == 123)

// Test trait method renaming (trait itself keeps original name)
val traitImpl = createTraitImpl(5)
assert(traitImpl.renamedTraitMethod(10) == 50) // 10 * 5 = 50
