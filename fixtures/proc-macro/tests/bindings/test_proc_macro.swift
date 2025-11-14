/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import proc_macro

let one = makeOne(inner: 123)
assert(one.inner == 123)
assert(oneInnerByRef(one: one) == 123)
assert(one.getInnerValue() == 123)

let two = Two(a: "a")
assert(takeTwo(two: two) == "a")

let rwb = RecordWithBytes(someBytes: Data([1, 2, 3]))
assert(takeRecordWithBytes(rwb: rwb) == Data([1, 2, 3]))

var obj = Object()
obj = Object.namedCtor(arg: 1)
assert(obj.isHeavy() == .uncertain)
let obj2 = Object()
assert(obj.isOtherHeavy(other: obj2) == .uncertain)

let traitImpl = obj.getTrait(inc: nil)
assert(traitImpl.concatStrings(a: "foo", b: "bar") == "foobar")
assert(obj.getTrait(inc: traitImpl).concatStrings(a: "foo", b: "bar") == "foobar")
assert(concatStringsByRef(t: traitImpl, a: "foo", b: "bar") == "foobar")

let traitImpl2 = obj.getTraitWithForeign(inc: nil)
assert(traitImpl2.name() == "RustTraitImpl")
assert(obj.getTraitWithForeign(inc: traitImpl2).name() == "RustTraitImpl")

assert(enumIdentity(value: .true) == .true)

// just make sure this works / doesn't crash
let three = Three(obj: obj)

assert(makeZero().inner == "ZERO")
assert(makeRecordWithBytes().someBytes == Data([0, 1, 2, 3, 4]))
assert(join(parts: ["a", "b", "c"], sep: ":") == "a:b:c")

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

// Defaults

let recordWithDefaults = RecordWithDefaults(noDefaultString: "Test")
assert(recordWithDefaults.noDefaultString == "Test")
assert(recordWithDefaults.boolean == true)
assert(recordWithDefaults.integer == 42)
assert(recordWithDefaults.floatVar == 4.2)
assert(recordWithDefaults.vec == [])
assert(recordWithDefaults.optVec == nil)
assert(recordWithDefaults.optInteger == 42)
assert(recordWithDefaults.customInteger == 42)

// Implicit defaults
let recordWithImplicitDefaults = RecordWithImplicitDefaults()
assert(recordWithImplicitDefaults.boolean == false)
assert(recordWithImplicitDefaults.int8 == 0)
assert(recordWithImplicitDefaults.uint8 == 0)
assert(recordWithImplicitDefaults.int16 == 0)
assert(recordWithImplicitDefaults.uint16 == 0)
assert(recordWithImplicitDefaults.int32 == 0)
assert(recordWithImplicitDefaults.uint32 == 0)
assert(recordWithImplicitDefaults.int64 == 0)
assert(recordWithImplicitDefaults.uint64 == 0)
assert(recordWithImplicitDefaults.afloat == 0.0)
assert(recordWithImplicitDefaults.adouble == 0.0)
assert(recordWithImplicitDefaults.vec == [])
assert(recordWithImplicitDefaults.map == [:])
assert(recordWithImplicitDefaults.someBytes == Data([]))
assert(recordWithImplicitDefaults.optInt32 == nil)
assert(recordWithImplicitDefaults.customInteger == 0)

// defaults in function args
assert(doubleWithDefault() == 42)
assert(sumWithDefault(num1: 1) == 1)
assert(sumWithDefault(num1: 1, num2: 2) == 3)

let objWithDefaults = ObjectWithDefaults()
assert(objWithDefaults.addToNum() == 42)
assert(objWithDefaults.addToImplicitNum() == 30)
assert(objWithDefaults.addToImplicitNum(other: 1) == 31)

// Traits

final class SwiftTestCallbackInterface : TestCallbackInterface {
    func doNothing() { }

    func add(a: UInt32, b: UInt32) -> UInt32 {
        return a + b;
    }

    func `optional`(a: Optional<UInt32>) -> UInt32 {
        return a ?? 0;
    }

    func withBytes(rwb: RecordWithBytes) -> Data {
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
        return h.takeError(e: BasicError.InvalidInput)
    }

    func getOtherCallbackInterface() -> OtherCallbackInterface {
        SwiftTestCallbackInterface2()
    }
}

final class SwiftTestCallbackInterface2 : OtherCallbackInterface {
    func multiply(a: UInt32, b: UInt32) -> UInt32 {
        return a * b;
    }
}

callCallbackInterface(cb: SwiftTestCallbackInterface())

assert(getMixedEnum(v: nil) == .int(1))
assert(getMixedEnum(v: MixedEnum.none) == .none)
assert(getMixedEnum(v: MixedEnum.string("hello")) == .string("hello"))
switch MixedEnum.string("hello") {
    case let .string(s):
        assert(s == "hello")
    default:
        assert(false)
}

switch MixedEnum.both("hello", 1) {
    case let .both(s, i):
        assert(s == "hello")
        assert(i == 1)
    default:
        assert(false)
}

switch MixedEnum.all(s: "string", i: 2) {
    case let .all(s, i):
        assert(s == "string")
        assert(i == 2)
    default:
        assert(false)
}

assert(getMixedEnum(v: MixedEnum.vec(["hello"])) == .vec(["hello"]))

// check autogenerated CaseIterable
assert(MaybeBool.allCases.map({ "\($0)" }).joined(separator: ", ") == "true, false, uncertain")
assert(ReprU8.allCases.map({ "\($0)" }).joined(separator: ", ") == "one, three")
assert(SimpleError.allCases.map({ "\($0)" }).joined(separator: ", ") == "A, B")
