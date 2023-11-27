/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture_foreign_executor.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking


val coroutineScope = CoroutineScope(Dispatchers.IO)
// Test scheduling calls with no delay
runBlocking {
    val tester = ForeignExecutorTester(coroutineScope)
    launch {
        tester.scheduleTest(0U)
    }
    delay(100L)
    val result = tester.getLastResult() ?: throw RuntimeException("ForeignExecutorTester.getLastResult() returned null")
    assert(result.callHappenedInDifferentThread)
    assert(result.delayMs <= 100U)
    tester.close()
}

// Test scheduling calls with a delay and using the newFromSequence constructor
runBlocking {
    val tester = ForeignExecutorTester.newFromSequence(listOf(coroutineScope))
    launch {
        tester.scheduleTest(100U)
    }
    delay(200L)
    val result = tester.getLastResult() ?: throw RuntimeException("ForeignExecutorTester.getLastResult() returned null")
    assert(result.callHappenedInDifferentThread)
    assert(result.delayMs >= 100U)
    assert(result.delayMs <= 200U)
    tester.close()
}
