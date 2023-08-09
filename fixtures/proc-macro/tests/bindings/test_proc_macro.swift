/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_proc_macro

let one = makeOne(inner: 123)
assert(one.inner == 123)

let two = Two(a: "a")
assert(takeTwo(two: two) == "a")

let rwb = RecordWithBytes(someBytes: [1, 2, 3])
assert(takeRecordWithBytes(rwb: rwb) == [1, 2, 3])

var obj = Object()
obj = Object.namedCtor(arg: 1)
assert(obj.isHeavy() == .uncertain)

assert(enumIdentity(value: .true) == .true)

// just make sure this works / doesn't crash
let three = Three(obj: obj)

assert(makeZero().inner == "ZERO")
assert(makeRecordWithBytes().someBytes == [0, 1, 2, 3, 4])

do {
    try alwaysFails()
    fatalError("alwaysFails should have thrown")
} catch BasicError.OsError {
}

try! obj.doStuff(times: 5)

do {
    try obj.doStuff(times: 0)
    fatalError("doStuff should throw if its argument is 0")
} catch FlatError.InvalidInput {
}

struct SomeOtherError: Error { }

class SwiftTestCallbackInterface : TestCallbackInterface {
    func doNothing() { }

    func add(a: UInt32, b: UInt32) -> UInt32 {
        return a + b;
    }

    func `optional`(a: Optional<UInt32>) -> UInt32 {
        return a ?? 0;
    }

    func withBytes(rwb: RecordWithBytes) -> [UInt8] {
        return rwb.someBytes
    }

    func tryParseInt(value: String) throws -> UInt32 {
        if (value == "force-unexpected-error") {
            // raise an error that's not expected
            throw SomeOtherError()
        }
        let parsed = UInt32(value)
        if parsed != nil {
            return parsed!
        } else {
            throw BasicError.InvalidInput
        }
    }

    func callbackHandler(h: Object) -> UInt32 {
        var v = h.takeError(e: BasicError.InvalidInput)
        return v
    }
}

testCallbackInterface(cb: SwiftTestCallbackInterface())
