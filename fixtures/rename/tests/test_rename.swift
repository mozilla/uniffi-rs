/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_fixture_rename

// Test renamed types - should be accessible by their custom names
let record = RenamedRecord(renamedField: 42)
assert(record.renamedField == 42)

let enum1 = RenamedEnum.renamedVariant
let enum2 = RenamedEnum.record(record)

// Test renamed function
let result = renamedFunction(record: record)
if case .record(let recordResult) = result {
    assert(recordResult.renamedField == 42)
}

// Test renamed object with renamed constructor and method
let obj = RenamedObject.renamedConstructor(value: 123)
let methodResult = obj.renamedMethod()
assert(methodResult == 123)

// Test trait method renaming (trait itself keeps original name)
let traitImpl = createTraitImpl(multiplier: 5)
assert(traitImpl.renamedTraitMethod(value: 10) == 50) // 10 * 5

// Test TOML-based renaming for Swift
// These use the Swift-specific renames from uniffi.toml

// Test renamed record via TOML
let swiftRecord = SwiftRecord(swiftItem: 100)
assert(swiftRecord.swiftItem == 100)

// Test renamed enum via TOML
let swiftEnum1 = SwiftEnum.swiftVariantA
let swiftEnum2 = SwiftEnum.swiftRecord(swiftRecord)
let swiftEnum3 = SwiftEnumWithFields.swiftVariantA(swiftInt: 1)

// Test renamed function with renamed argument via TOML
do {
    let swiftResult = try swiftFunction(swiftRecord: swiftRecord)
    if case .swiftRecord(let recordResult) = swiftResult {
        assert(recordResult.swiftItem == 100)
    }
} catch let error as SwiftError {
    if case .SwiftSimple = error {
        // Expected for nil input
    }
}

// Test that we can also get an error from the function
do {
    let _ = try swiftFunction(swiftRecord: nil)
    fatalError("Should have thrown an error")
} catch let error as SwiftError {
    if case .SwiftSimple = error {
        // Expected
    } else {
        fatalError("Wrong error type")
    }
}

// Test renamed object with renamed method and argument via TOML
let swiftObj = SwiftObject(value: 200)
do {
    let swiftMethodResult = try swiftObj.swiftMethod(swiftArg: 50)
    assert(swiftMethodResult == 250) // 200 + 50
} catch {
    fatalError("Should not have thrown an error")
}

// Test renamed trait with renamed method via TOML
let swiftTraitImpl = createBindingTraitImpl(multiplier: 3)
assert(swiftTraitImpl.swiftTraitMethod(value: 7) == 21) // 7 * 3
