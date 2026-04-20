/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import org.mozilla.uniffi.benchmarks.*
import kotlin.ByteArray
import kotlin.experimental.xor
import kotlin.system.measureNanoTime

// Create objects to use in the tests.  This way the benchmarks don't include the time needed to
// construct these objects.
object TestData {
    val testLargeString1 = "a".repeat(2048)
    val testLargeString2 = "b".repeat(1500)
    val testRec1 = TestRecord(a = -1, b = 1.toULong(), c = 1.5)
    val testRec2 = TestRecord(a = -2, b = 2.toULong(), c = 4.5)
    val testLargeRec1 = TestLargeRecord(a = 1, b = 2, c = 3, d = 4, e = 1.0f, f = 2.0, g = true)
    val testLargeRec2 = TestLargeRecord(a = -1, b = -2, c = -3, d = -4, e = -1.0f, f = -2.0, g = false)
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
    val testBytes = ByteArray(256) { it.toByte() }
    val testPrimitiveList = (0..1024).map { it.toUInt() }
    val testRecordList = (0..1024).map { TestRecord(it, it.toULong() * 2uL, it.toDouble() / 2.0) }
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

    override fun largeRecords(a: TestLargeRecord, b: TestLargeRecord): TestLargeRecord {
        return TestLargeRecord(
            a=(a.a + b.a).toByte(),
            b=(a.b + b.b).toShort(),
            c=a.c + b.c,
            d=a.d + b.d,
            e=a.e + b.e,
            f=a.f + b.f,
            g=a.g && b.g,
        )
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

    override fun optionals(a: UInt?, b: Boolean?, c: String?): UInt {
        var sum = 0u;
        if (a != null) {
            sum += a;
        }
        if (b == true) {
            sum *= 2u;
        }
        if (c != null) {
            sum += c.length.toUInt()
        }
        return sum.toUInt()
    }

    override fun bytes(v: ByteArray): ByteArray {
        return v
    }

    override fun vecSmall(a: List<UInt>, b: List<UInt>): List<UInt> {
        return a + b
    }

    override fun vecPrimitives(v: List<UInt>): List<UInt> {
        return v
    }

    override fun vecRecords(v: List<TestRecord>): List<TestRecord> {
        return v
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

            TestCase.LARGE_RECORDS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseLargeRecords(TestData.testLargeRec1, TestData.testLargeRec2)
                }
            }

            TestCase.ENUMS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseEnums(TestData.testEnum1, TestData.testEnum2)
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

            TestCase.OPTIONALS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseOptionals(10u, null, "testing-123")
                }
            }

            TestCase.BYTES -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseBytes(TestData.testBytes)
                }
            }

            TestCase.VEC_SMALL -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseVecSmall(TestData.testVec1, TestData.testVec2)
                }
            }

            TestCase.VEC_PRIMITIVES -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseVecPrimitives(TestData.testPrimitiveList)
                }
            }

            TestCase.VEC_RECORDS -> measureNanoTime {
                for (i in 0UL..count) {
                    testCaseVecRecords(TestData.testRecordList)
                }
            }

            TestCase.METHODS -> measureNanoTime {
                for (i in 0UL..count) {
                    TestData.testInterface.noopMethod()
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
