/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#if canImport(benchmarks)
    import benchmarks
#endif

#if os(Linux)
    import Glibc
#else
    import Darwin.C
#endif

// Create objects to use in the tests.  This way the benchmarks don't include the time needed to
// construct these objects.

let TEST_LARGE_STRING1 = String(repeating: "a", count: 2048)
let TEST_LARGE_STRING2 = String(repeating: "b", count: 1500)
let TEST_REC1 = TestRecord(a: -1, b: 1, c: 1.5)
let TEST_REC2 = TestRecord(a: -2, b: 2, c: 4.5)
let TEST_ENUM1 = TestEnum.one(a: -1, b: 0)
let TEST_ENUM2 = TestEnum.two(c: 1.5)
let TEST_VEC1: [UInt32] = [0, 1]
let TEST_VEC2: [UInt32] = [2, 4, 6]
let TEST_MAP1: [UInt32: UInt32] = [ 0: 1, 1: 2 ]
let TEST_MAP2: [UInt32: UInt32] = [ 2: 4 ]
let TEST_INTERFACE = TestInterface()
let TEST_INTERFACE2 = TestInterface()
let TEST_TRAIT_INTERFACE = makeTestTraitInterface()
let TEST_TRAIT_INTERFACE2 = makeTestTraitInterface()
let TEST_NESTED_DATA1 = NestedData(
    a: [TestRecord(a: -1, b: 1, c: 1.5)],
    b: [["one", "two"], ["three"]],
    c: [
        "one": TestEnum.one(a: -1, b: 1),
        "two": TestEnum.two(c: 0.5),
    ]
)
let TEST_NESTED_DATA2 = NestedData(
    a: [TestRecord(a: -2, b: 2, c: 4.5)],
    b: [["four", "five"]],
    c: [
        "two": TestEnum.two(c: -0.5),
    ]
)

final class TestCallbackObj: TestCallbackInterface {
    func callOnly() {
    }

    func primitives(a: UInt8, b: Int32) -> Float64 {
        Float64(a) + Float64(b)
    }

    func strings(a: String, b: String) -> String {
        a + b
    }

    func largeStrings(a: String, b: String) -> String {
        a + b
    }

    func records(a: TestRecord, b: TestRecord) -> TestRecord {
        return TestRecord(
            a: a.a + b.a,
            b: a.b + b.b,
            c: a.c + b.c
        )
    }

    func enums(a: TestEnum, b: TestEnum) -> TestEnum {
        let aSum = switch a {
        case .one(let a, let b): Float64(a) + Float64(b)
        case .two(let c): c
        }
        let bSum = switch b {
        case .one(let a, let b): Float64(a) + Float64(b)
        case .two(let c): c
        }
        return TestEnum.two(c: aSum + bSum)
    }

    func vecs(a: [UInt32], b: [UInt32]) -> [UInt32] {
        return a + b
    }

    func hashMaps(
        a: Dictionary<UInt32, UInt32>,
        b: Dictionary<UInt32, UInt32>
    ) -> Dictionary<UInt32, UInt32> {
        return a.merging(b) { (_, new) in new }
    }

    func interfaces(a: TestInterface, b: TestInterface) -> TestInterface {
        // Perform some silliness to make sure Swift needs to access both `a` and `b`
        if (a === b) {
            return a
        } else {
            return b
        }
    }

    func traitInterfaces(
        a: TestTraitInterface,
        b: TestTraitInterface
    ) -> TestTraitInterface {
        // Perform some silliness to make sure Swift needs to access both `a` and `b`
        if (a === b) {
            return a
        } else {
            return b
        }
    }

    func nestedData(a: NestedData, b: NestedData) -> NestedData {
        return NestedData(
            a: a.a + b.a,
            b: a.b + b.b,
            c: a.c.merging(b.c) { (_, new) in new }
        )
    }

    func errors() throws -> UInt32 {
        throw TestError.Two
    }

    func runTest(testCase: TestCase, count: UInt64) -> UInt64 {
        let start: clock_t
        switch testCase {
        case TestCase.callOnly:
            start = clock()
            for _ in 0...count {
                testCaseCallOnly()
            }

        case TestCase.primitives:
            start = clock()
            for _ in 0...count {
                let _ = testCasePrimitives(a: 0, b: 1)
            }

        case TestCase.strings:
            start = clock()
            for _ in 0...count {
                let _ = testCaseStrings(a: "a", b: "b")
            }

        case TestCase.largeStrings:
            start = clock()
            for _ in 0...count {
                let _ = testCaseStrings(a: TEST_LARGE_STRING1, b: TEST_LARGE_STRING2)
            }

        case TestCase.records:
            start = clock()
            for _ in 0...count {
                let _ = testCaseRecords(a: TEST_REC1, b: TEST_REC2)
            }

        case TestCase.enums:
            start = clock()
            for _ in 0...count {
                let _ = testCaseEnums(a: TEST_ENUM1, b: TEST_ENUM2)
            }

        case TestCase.vecs:
            start = clock()
            for _ in 0...count {
                let _ = testCaseVecs(a: TEST_VEC1, b: TEST_VEC2)
            }

        case TestCase.hashmaps:
            start = clock()
            for _ in 0...count {
                let _ = testCaseHashmaps(a: TEST_MAP1, b: TEST_MAP2)
            }

        case TestCase.interfaces:
            start = clock()
            for _ in 0...count {
                let _ = testCaseInterfaces(a: TEST_INTERFACE, b: TEST_INTERFACE2)
            }

        case TestCase.traitInterfaces:
            start = clock()
            for _ in 0...count {
                let _ = testCaseTraitInterfaces(a: TEST_TRAIT_INTERFACE, b: TEST_TRAIT_INTERFACE2)
            }

        case TestCase.nestedData:
            start = clock()
            for _ in 0...count {
                let _ = testCaseNestedData(a: TEST_NESTED_DATA1, b: TEST_NESTED_DATA2)
            }

        case TestCase.errors:
            start = clock()
            for _ in 0...count {
                let _ = try? testCaseErrors()
            }
        }

        let end = clock()
        return UInt64((end - start) * 1000000000 / CLOCKS_PER_SEC)
    }
}

runBenchmarks(language: "swift", cb: TestCallbackObj())
