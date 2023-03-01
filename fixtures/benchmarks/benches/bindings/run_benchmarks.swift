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

class TestCallbackObj: TestCallbackInterface {
    func testMethod(a: Int32, b: Int32, data: TestData) -> String {
        return data.bar
    }

    func testVoidReturn(a: Int32, b: Int32, data: TestData) {
    }

    func testNoArgsVoidReturn() {
    }

    func runTest(testCase: TestCase, count: UInt64) -> UInt64 {
        let data = TestData(foo: "StringOne", bar: "StringTwo")
        let start = clock()
        switch testCase {
        case TestCase.function:
            for _ in 0...count {
                testFunction(a: 10, b: 20, data: data)
            }
        case TestCase.voidReturn:
            for _ in 0...count {
                testVoidReturn(a: 10, b: 20, data: data)
            }

        case TestCase.noArgsVoidReturn:
            for _ in 0...count {
                testNoArgsVoidReturn()
            }
        }
        let end = clock()
        return UInt64((end - start) * 1000000000 / CLOCKS_PER_SEC)
    }
}

runBenchmarks(languageName: "swift", cb: TestCallbackObj())
