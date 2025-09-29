/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.benchmarks.*
import kotlin.system.measureNanoTime

class TestCallbackObj : TestCallbackInterface {
    override fun method(a: Int, b: Int, data: TestData): String {
        return data.bar;
    }

    override fun methodWithVoidReturn(a: Int, b: Int, data: TestData) {
    }

    override fun methodWithNoArgsAndVoidReturn() {
    }

    override fun runTest(testCase: TestCase, count: ULong): ULong {
        return when (testCase) {
            TestCase.FUNCTION -> measureNanoTime {
                val data = TestData("StringOne", "StringTwo")
                for (i in 0UL..count) {
                    testFunction(10, 20, data)
                }
            }
            TestCase.VOID_RETURN -> measureNanoTime {
                for (i in 0UL..count) {
                    testVoidReturn(10, 20)
                }
            }
            TestCase.NO_ARGS_VOID_RETURN -> measureNanoTime {
                for (i in 0UL..count) {
                    testNoArgsVoidReturn()
                }
            }
        }.toULong()
    }
}

runBenchmarks("kotlin", TestCallbackObj())
