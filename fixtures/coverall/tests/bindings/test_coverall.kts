/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*

import uniffi.coverall.*

// Test some_dict().
// TODO: use an actual test runner.

val d = createSomeDict();
assert(d.text == "text");
assert(d.maybeText == "maybe_text");
assert(d.aBool);
assert(d.maybeABool == false);
assert(d.unsigned8 == 1.toUByte())
assert(d.maybeUnsigned8 == 2.toUByte())
assert(d.unsigned64 == 18446744073709551615UL)
assert(d.maybeUnsigned64 == 0UL)
assert(d.signed8 == 8.toByte())
assert(d.maybeSigned8 == 0.toByte())
assert(d.signed64 == 9223372036854775807L)
assert(d.maybeSigned64 == 0L)

// floats should be "close enough".
fun Float.almostEquals(other: Float) = Math.abs(this - other) < 0.000001
fun Double.almostEquals(other: Double) = Math.abs(this - other) < 0.000001

assert(d.float32.almostEquals(1.2345F))
assert(d.maybeFloat32!!.almostEquals(22.0F/7.0F))
assert(d.float64.almostEquals(0.0))
assert(d.maybeFloat64!!.almostEquals(1.0))

// This tests that the UniFFI-generated scaffolding doesn't introduce any unexpected locking.
// We have one thread busy-wait for a some period of time, while a second thread repeatedly
// increments the counter and then checks if the object is still busy. The second thread should
// not be blocked on the first, and should reliably observe the first thread being busy.
// If it does not, that suggests UniFFI is accidentally serializing the two threads on access
// to the shared counter object.

ThreadsafeCounter().use { counter ->
    val executor = Executors.newFixedThreadPool(3)
    try {
        val busyWaiting: Future<Unit> = executor.submit(Callable {
            // 300 ms should be long enough for the other thread to easily finish
            // its loop, but not so long as to annoy the user with a slow test.
            counter.busyWait(300)
        })
        val incrementing: Future<Int> = executor.submit(Callable {
            var count = 0
            for (n in 1..100) {
                // We exect most iterations of this loop to run concurrently
                // with the busy-waiting thread.
                count = counter.incrementIfBusy()
            }
            count
        })

        busyWaiting.get()
        val count = incrementing.get()
        assert(count > 0) { "Counter doing the locking: incrementIfBusy=$count" }
    } finally {
        executor.shutdown()
    }
}
