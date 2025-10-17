/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package org.mozilla.uniffi.benchmarks

import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.RadioGroup
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import kotlin.system.measureNanoTime

class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("uniffi_benchmarks")
        }
    }

    private lateinit var resultTextView: TextView
    private lateinit var editIterationCount: EditText
    private lateinit var radioGroupCallType: RadioGroup
    private lateinit var radioGroupTestCase: RadioGroup
    private val testCallback = TestCallbackImpl()
    private val results = StringBuilder()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        resultTextView = findViewById(R.id.resultTextView)
        editIterationCount = findViewById(R.id.editIterationCount)
        radioGroupCallType = findViewById(R.id.radioGroupCallType)
        radioGroupTestCase = findViewById(R.id.radioGroupTestCase)

        findViewById<Button>(R.id.btnRun).setOnClickListener {
            runBenchmark { runSelectedBenchmark() }
        }

        findViewById<Button>(R.id.btnClear).setOnClickListener {
            clearResults()
        }
    }

    private fun getIterationCount(): Int {
        return editIterationCount.text.toString().toIntOrNull()?.coerceAtLeast(1) ?: 10000
    }

    private fun isCallbackSelected(): Boolean {
        return radioGroupCallType.checkedRadioButtonId == R.id.radioCallback
    }

    private fun getSelectedTestCase(): TestCase? {
        return when (radioGroupTestCase.checkedRadioButtonId) {
            R.id.radioCallOnly -> TestCase.CALL_ONLY
            R.id.radioPrimitives -> TestCase.PRIMITIVES
            R.id.radioStrings -> TestCase.STRINGS
            R.id.radioRecords -> TestCase.RECORDS
            R.id.radioEnums -> TestCase.ENUMS
            R.id.radioVecs -> TestCase.VECS
            R.id.radioHashmaps -> TestCase.HASHMAPS
            R.id.radioInterfaces -> TestCase.INTERFACES
            R.id.radioTraitInterfaces -> TestCase.TRAIT_INTERFACES
            R.id.radioNestedData -> TestCase.NESTED_DATA
            R.id.radioErrors -> TestCase.ERRORS
            else -> null
        }
    }

    private fun runBenchmark(benchmark: () -> String) {
        // Disable buttons during benchmark
        setButtonsEnabled(false)

        Thread {
            val result = try {
                benchmark()
            } catch (e: Exception) {
                "✗ Error: ${e.message}\n${e.stackTraceToString()}\n\n"
            }

            runOnUiThread {
                appendResult(result)
                setButtonsEnabled(true)
            }
        }.start()
    }

    private fun formatBenchmarkResult(name: String, iterations: Int, timeNanos: ULong): String {
        if (iterations == 0) {
            return "0 iterations"
        }
        val perCall = timeNanos.toDouble() / iterations.toDouble()
        val time = timeNanos.toDouble() / 1000000000.0
        return "$name\n" +
               "Iterations: $iterations\n" +
               "Total: ${"%.3f".format(time)}s\n" +
               "Avg: ${"%.3f".format(perCall)}ns"
    }

    private fun runSelectedBenchmark(): String {
        val isCallback = isCallbackSelected()
        val testCase = getSelectedTestCase()

        if (testCase == null) {
            return "No test case selected"
        }

        val iterations = getIterationCount()

        val callTypeStr = if (isCallback) "Callback" else "Function"
        val testCaseStr = when (testCase) {
            TestCase.CALL_ONLY -> "Call Only"
            TestCase.PRIMITIVES -> "Primitives"
            TestCase.STRINGS -> "Strings"
            TestCase.RECORDS -> "Records"
            TestCase.ENUMS -> "Enums"
            TestCase.VECS -> "Vecs"
            TestCase.HASHMAPS -> "Hashmaps"
            TestCase.INTERFACES -> "Interfaces"
            TestCase.TRAIT_INTERFACES -> "Trait Interfaces"
            TestCase.NESTED_DATA -> "Nested Data"
            TestCase.ERRORS -> "Errors"
        }
        val benchmarkName = "$callTypeStr - $testCaseStr"

        return if (isCallback) {
            runCallbackBenchmark(testCase, iterations, benchmarkName)
        } else {
            runFunctionBenchmark(testCase, iterations, benchmarkName)
        }
    }

    private fun runFunctionBenchmark(testCase: TestCase, iterations: Int, name: String): String {
        val time = testCallback.runTest(testCase, iterations.toULong())
        return formatBenchmarkResult(name, iterations, time)
    }

    private fun runCallbackBenchmark(testCase: TestCase, iterations: Int, name: String): String {
        val time = measureNanoTime {
            runCallbackTest(testCallback, testCase, iterations.toULong())
        }
        return formatBenchmarkResult(name, iterations, time.toULong())
    }

    private fun appendResult(result: String) {
        results.append(result)
        resultTextView.text = results.toString()
    }

    private fun clearResults() {
        results.clear()
        resultTextView.text = "Select options and click Run Benchmark..."
    }

    private fun setButtonsEnabled(enabled: Boolean) {
        findViewById<Button>(R.id.btnRun).isEnabled = enabled
        findViewById<Button>(R.id.btnClear).isEnabled = enabled
        radioGroupCallType.isEnabled = enabled
        radioGroupTestCase.isEnabled = enabled
        editIterationCount.isEnabled = enabled
    }

    private class TestCallbackImpl : TestCallbackInterface {
        override fun callOnly() {
        }

        override fun primitives(a: UByte, b: Int): Double {
            return a.toDouble() + b.toDouble()
        }

        override fun strings(a: String, b: String): String {
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
}

// Create objects to use in the tests.  This way the benchmarks don't include the time needed to
// construct these objects.
object TestData {
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
