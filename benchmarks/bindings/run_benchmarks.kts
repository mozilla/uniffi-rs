/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import org.mozilla.uniffi.benchmarks.*
import kotlin.system.measureNanoTime

// Create objects to use in the tests.  This way the benchmarks don't include the time needed to
// construct these objects.
object TestData {
    val testLargeString1 = "a".repeat(2048)
    val testLargeString2 = "b".repeat(1500)
    val testRec1 = TestRecord(a = -1, b = 1.toULong(), c = 1.5)
    val testRec2 = TestRecord(a = -2, b = 2.toULong(), c = 4.5)
    val testEnum1 = TestEnum.One(a = -1, b = 0.toULong())
    val testEnum2 = TestEnum.Two(c = 1.5)
    val testVec1 = listOf(0.toUInt(), 1.toUInt())
    val testVec2 = listOf(2.toUInt(), 4.toUInt(), 6.toUInt())
    val testMap1 = mapOf(0.toUInt() to 1.toUInt(), 1.toUInt() to 2.toUInt())
    val testMap2 = mapOf(2.toUInt() to 4.toUInt())
    val testInterface = TestInterface()
    val testInterface2 = TestInterface()
    val testTraitInterface = makeTestTraitInterface()
    val testTraitInterface2 = makeTestTraitInterface()
    val testNestedData1 = NestedData(
        a = listOf(TestRecord(a = -1, b = 1.toULong(), c = 1.5)),
        b = listOf(listOf("one", "two"), listOf("three")),
        c = mapOf(
            "one" to TestEnum.One(a = -1, b = 1.toULong()),
            "two" to TestEnum.Two(c = 0.5),
        )
    )
    val testNestedData2 = NestedData(
        a = listOf(TestRecord(a = -2, b = 2.toULong(), c = 4.5)),
        b = listOf(listOf("four", "five")),
        c = mapOf(
            "two" to TestEnum.Two(c = -0.5),
        )
    )
}

class TestCallbackObj : TestCallbackInterface {
    override fun callOnly() {
    }

    override fun primitives(a: UByte, b: Int): Double {
        return a.toDouble() + b.toDouble()
    }

    override fun strings(a: String, b: String): String {
        return a + b
    }

    override fun largeStrings(a: String, b: String): String {
        return a + b
    }

    override fun records(a: TestRecord, b: TestRecord): TestRecord {
        return TestRecord(a=a.a + b.a, b=a.b + b.b, c=a.c + b.c)
    }

    override fun enums(a: TestEnum, b: TestEnum): TestEnum {
        val aSum = when (a) {
            is TestEnum.One -> a.a.toDouble() + a.b.toDouble()
            is TestEnum.Two -> a.c
        }
        val bSum = when (b) {
            is TestEnum.One -> b.a.toDouble() + b.b.toDouble()
            is TestEnum.Two -> b.c
        }
        return TestEnum.Two(aSum + bSum)
    }

    override fun vecs(a: List<UInt>, b: List<UInt>): List<UInt> {
        return a + b
    }

    override fun hashMaps(
        a: Map<UInt, UInt>,
        b: Map<UInt, UInt>
    ): Map<UInt, UInt> {
        return a + b
    }

    override fun interfaces(a: TestInterface, b: TestInterface): TestInterface {
        // Perform some silliness to make sure Kotlin needs to access both `a` and `b`
        return if (a == b) {
            a
        } else {
            b
        }
    }

    override fun traitInterfaces(
        a: TestTraitInterface,
        b: TestTraitInterface
    ): TestTraitInterface {
        // Perform some silliness to make sure Kotlin needs to access both `a` and `b`
        return if (a == b) {
            a
        } else {
            b
        }
    }

    override fun nestedData(a: NestedData, b: NestedData): NestedData {
        // Perform some silliness to make sure Kotlin need to access both `a` and `b`
        return if (a == b) {
            a
        } else {
            b
        }

        return NestedData(
            a = a.a + b.a,
            b = a.b + b.b,
            c = a.c + b.c,
        )
    }

    override fun errors(): UInt {
        throw TestException.Two()
    }

    override fun runTest(testCase: TestCase, count: ULong): ULong {
        return when (testCase) {
            TestCase.CALL_ONLY -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseCallOnly()
                }
            }

            TestCase.PRIMITIVES -> measureNanoTime {
                for (i in 0UL..count) {
                    testCasePrimitives(0.toUByte(), 1)
                }
            }

            TestCase.STRINGS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseStrings("a", "b")
                }
            }

            TestCase.LARGE_STRINGS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseLargeStrings(
                        TestData.testLargeString1,
                        TestData.testLargeString2
                    )
                }
            }

            TestCase.RECORDS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseRecords(TestData.testRec1, TestData.testRec2)
                }
            }

            TestCase.ENUMS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseEnums(TestData.testEnum1, TestData.testEnum2)
                }
            }

            TestCase.VECS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseVecs(TestData.testVec1, TestData.testVec2)
                }
            }

            TestCase.HASHMAPS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseHashmaps(TestData.testMap1, TestData.testMap2)
                }
            }

            TestCase.INTERFACES -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseInterfaces(TestData.testInterface, TestData.testInterface2)
                }
            }

            TestCase.TRAIT_INTERFACES -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseTraitInterfaces(TestData.testTraitInterface, TestData.testTraitInterface2)
                }
            }

            TestCase.NESTED_DATA -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseNestedData(TestData.testNestedData1, TestData.testNestedData2)
                }
            }

            TestCase.ERRORS -> measureNanoTime {
                for (i in 0UL..count) {
                    try {
                        testCaseErrors()
                    } catch (e: Exception) {
                        // ignore errors, they're expected
                    }
                }
            }
        }.toULong()
    }
}

runBenchmarks("kotlin", TestCallbackObj())
