/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripOptionU8(a: 67) == 67);
assert(roundtripOptionU8(a: nil) == nil);
assert(roundtripOptionI8(a: 67) == 67);
assert(roundtripOptionI8(a: nil) == nil);
assert(roundtripOptionU16(a: 67) == 67);
assert(roundtripOptionU16(a: nil) == nil);
assert(roundtripOptionI16(a: 67) == 67);
assert(roundtripOptionI16(a: nil) == nil);
assert(roundtripOptionU32(a: 67) == 67);
assert(roundtripOptionU32(a: nil) == nil);
assert(roundtripOptionI32(a: 67) == 67);
assert(roundtripOptionI32(a: nil) == nil);
assert(roundtripOptionU64(a: 67) == 67);
assert(roundtripOptionU64(a: nil) == nil);
assert(roundtripOptionI64(a: 67) == 67);
assert(roundtripOptionI64(a: nil) == nil);
assert(roundtripOptionF32(a: 67.0) == 67.0);
assert(roundtripOptionF32(a: nil) == nil);
assert(roundtripOptionF64(a: 67.0) == 67.0);
assert(roundtripOptionF64(a: nil) == nil);
assert(roundtripOptionBool(a: true) == true);
assert(roundtripOptionBool(a: nil) == nil);
assert(roundtripOptionString(a: "test-string") == "test-string");
assert(roundtripOptionString(a: nil) == nil);
assert(roundtripOptionRec(a: CompoundTypesRec(a: 67)) == CompoundTypesRec(a: 67))
assert(roundtripOptionRec(a: nil) == nil);

assert(roundtripVecI8(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecU16(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecI16(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecU32(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecI32(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecU64(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecI64(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecF32(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecF64(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripVecBool(a: [true, false]) == [true, false]);
assert(roundtripVecString(a: ["test-string"]) == ["test-string"]);
assert(roundtripVecRec(a: [CompoundTypesRec(a: 67)]) == [CompoundTypesRec(a: 67)])

assert(roundtripHashMap(a: ["a": 1, "b": 2]) == ["a": 1, "b": 2])
assert(roundtripHashSet(a: ["a", "b", "c"]) == ["a", "b", "c"])
assert(roundtripHashMapU32Key(a: [1: 2, 2: 4]) == [1: 2, 2: 4])

assert(
    RecWithCompounds(
        a: EnumWithCompounds.a(nil),
        b: nil,
        c: [true, false],
        d: [
            "a": 10,
            "b": 20,
        ]
    ) == RecWithCompounds(
        a: EnumWithCompounds.a(nil),
        b: nil,
        c: [true, false],
        d: [
            "a": 10,
            "b": 20,
        ]
    )
)
assert(roundtripComplexCompound(a: [
    [
        "a": CompoundTypesComplexRec(a: 10, b: "Test", c: CompoundTypesEnum.a(100)),
        "b": CompoundTypesComplexRec(a: 20, b: "Test2", c: CompoundTypesEnum.b(a: 1.0, b: true))
    ]
]) == [
    [
        "a": CompoundTypesComplexRec(a: 10, b: "Test", c: CompoundTypesEnum.a(100)),
        "b": CompoundTypesComplexRec(a: 20, b: "Test2", c: CompoundTypesEnum.b(a: 1.0, b: true))
    ]
])
assert(roundtripComplexCompound(a: nil) == nil);
assert(roundtripComplexHashSet(a: [["a", "b"]]) == [["a", "b"]])
assert(roundtripComplexHashSet(a: nil) == nil)
