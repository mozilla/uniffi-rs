/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

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
assert(roundtripVecRec(a: [CollectionsRec(a: 67)]) == [CollectionsRec(a: 67)])

assert(roundtripHashMap(a: ["a": 1, "b": 2]) == ["a": 1, "b": 2])
assert(roundtripHashSet(a: ["a", "b", "c"]) == ["a", "b", "c"])
assert(roundtripHashMapU32Key(a: [1: 2, 2: 4]) == [1: 2, 2: 4])

assert(
    roundtripRecWithCollections(a: RecWithCollections(
            a: EnumWithCollections.a(nil),
            b: nil,
            c: [true, false],
            d: [
                "a": 10,
                "b": 20,
            ]
        )
    ) == RecWithCollections(
        a: EnumWithCollections.a(nil),
        b: nil,
        c: [true, false],
        d: [
            "a": 10,
            "b": 20,
        ]
    )
)
assert(roundtripVecHashSet(a: [["a", "b"]]) == [["a", "b"]])
assert(roundtripVecHashSet(a: nil) == nil)
assert(roundtripComplexCollectionType(a: [
    [
        "a": CollectionsComplexRec(a: 10, b: "Test", c: CollectionsEnum.a(100)),
        "b": CollectionsComplexRec(a: 20, b: "Test2", c: CollectionsEnum.b(a: 1.0, b: true))
    ]
]) == [
    [
        "a": CollectionsComplexRec(a: 10, b: "Test", c: CollectionsEnum.a(100)),
        "b": CollectionsComplexRec(a: 20, b: "Test2", c: CollectionsEnum.b(a: 1.0, b: true))
    ]
])
assert(roundtripComplexCollectionType(a: nil) == nil);
