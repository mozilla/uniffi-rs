/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package org.mozilla.uniffi.benchmarks

import android.os.Bundle
import android.widget.Button
import android.widget.EditText
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
    private val testCallback = TestCallbackImpl()
    private val results = StringBuilder()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        resultTextView = findViewById(R.id.resultTextView)
        editIterationCount = findViewById(R.id.editIterationCount)

        findViewById<Button>(R.id.btnFunctionCallNoArgsVoidReturn).setOnClickListener {
            runBenchmark { runTestCase("Function Call: no args void return", TestCase.NO_ARGS_VOID_RETURN) }
        }

        findViewById<Button>(R.id.btnFunctionCallVoidReturn).setOnClickListener {
            runBenchmark { runTestCase("Function Call: void return", TestCase.VOID_RETURN) }
        }

        findViewById<Button>(R.id.btnFunctionCallArgsAndReturn).setOnClickListener {
            runBenchmark { runTestCase("Function Call: args and return", TestCase.FUNCTION) }
        }

        findViewById<Button>(R.id.btnCallbackNoArgsVoidReturn).setOnClickListener {
            runBenchmark { runCallbackTestCase("Callback method: no args void return", CallbackTestCase.NO_ARGS_VOID_RETURN) }
        }

        findViewById<Button>(R.id.btnCallbackVoidReturn).setOnClickListener {
            runBenchmark { runCallbackTestCase("Callback method: void return", CallbackTestCase.VOID_RETURN) }
        }

        findViewById<Button>(R.id.btnCallbackArgsAndReturn).setOnClickListener {
            runBenchmark { runCallbackTestCase("Callback method: args and return", CallbackTestCase.ARGS_AND_RETURN) }
        }

        findViewById<Button>(R.id.btnClear).setOnClickListener {
            clearResults()
        }
    }

    private fun getIterationCount(): Int {
        return editIterationCount.text.toString().toIntOrNull()?.coerceAtLeast(1) ?: 50000
    }

    private fun runBenchmark(benchmark: () -> String) {
        // Disable all buttons during benchmark
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
        return "$name Benchmark\n" +
               "Iterations: $iterations\n" +
               "Time: ${"%.3f".format(time)}s (${"%.3f".format(perCall)}ns per call)\n\n"
    }

    private fun runTestCase(name: String, testCase: TestCase): String {
        // Warm up with 1000 iterations before measuring.
        testCallback.runTest(testCase, 1000.toULong())

        val iterations = getIterationCount()
        val time = testCallback.runTest(testCase, iterations.toULong())
        return formatBenchmarkResult(name, iterations, time)
    }

    private fun runCallbackTestCase(name: String, testCase: CallbackTestCase): String {
        // Warm up with 1000 iterations before measuring.
        runCallbackTest(testCallback, testCase, 1000.toULong())

        val iterations = getIterationCount()
        val time = measureNanoTime {
            runCallbackTest(testCallback, testCase, iterations.toULong())
        }
        return formatBenchmarkResult(name, iterations, time.toULong())
    }

    private fun runCallbackBenchmark(): String {
        val iterations = getIterationCount()
        val time = measureNanoTime {
            for (i in 0 until iterations) {
                testCallback.method(10, 20, TestData("foo", "bar"))
            }
        }
        return formatBenchmarkResult("Callback", iterations, time.toULong())
    }

    private fun appendResult(result: String) {
        results.append(result)
        resultTextView.text = results.toString()
    }

    private fun clearResults() {
        results.clear()
        resultTextView.text = "Click a button to run benchmarks..."
    }

    private fun setButtonsEnabled(enabled: Boolean) {
        findViewById<Button>(R.id.btnFunctionCallNoArgsVoidReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnFunctionCallVoidReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnFunctionCallArgsAndReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnCallbackNoArgsVoidReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnCallbackVoidReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnCallbackArgsAndReturn).isEnabled = enabled
        findViewById<Button>(R.id.btnClear).isEnabled = enabled
    }

    private class TestCallbackImpl : TestCallbackInterface {
        override fun method(a: Int, b: Int, data: TestData): String {
            return data.bar
        }

        override fun methodWithVoidReturn(a: Int, b: Int, data: TestData) {
            // No-op
        }

        override fun methodWithNoArgsAndVoidReturn() {
            // No-op
        }

        override fun runTest(testCase: TestCase, count: ULong): ULong {
            return when (testCase) {
                TestCase.FUNCTION -> measureNanoTime {
                    val data = TestData("StringOne", "StringTwo")
                    for (i in 0UL..count) {
                        testFunction(10, 20, data)
                    }
                }.toULong()
                TestCase.VOID_RETURN -> measureNanoTime {
                    for (i in 0UL..count) {
                        testVoidReturn(10, 20)
                    }
                }.toULong()
                TestCase.NO_ARGS_VOID_RETURN -> measureNanoTime {
                    for (i in 0UL..count) {
                        testNoArgsVoidReturn()
                    }
                }.toULong()
            }
        }
    }
}
