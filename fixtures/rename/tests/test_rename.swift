/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_fixture_rename

// Test renamed types - should be accessible by their custom names
let record = RenamedRecord(item: 42)
assert(record.item == 42)

let enum1 = RenamedEnum.variantA
let enum2 = RenamedEnum.record(record)

// Test renamed function
let result = renamedFunction(record: record)
if case .record(let recordResult) = result {
    assert(recordResult.item == 42)
}

// Test renamed object with renamed constructor and method
let obj = RenamedObject.renamedConstructor(value: 123)
let methodResult = obj.renamedMethod()
assert(methodResult == 123)

// Test trait method renaming (trait itself keeps original name)
let traitImpl = createTraitImpl(multiplier: 5)
assert(traitImpl.renamedTraitMethod(value: 10) == 50) // 10 * 5
