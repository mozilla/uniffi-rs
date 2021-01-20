/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*
import uniffi.threadsafe.*

/// We'd like to write objects so different methods can be executed on multiple 
/// threads at the same time.
///
/// We have two Rust implementations of the same object.
/// The objects can busy-wait, thus blocking the caller thread for some time.
/// On a different thread, the same object can increment a counter. The counter
/// should only increment when busy.
///
/// This function calls the busy-wait on one thread, and the incrementIfBusy 
/// multiple times on another.
fun countWhileBusy(busyWait: () -> Unit, incrementIfBusy: () -> Int): Int {
    val executor = Executors.newFixedThreadPool(3)
    return try {
        val busyWaiting: Future<Unit> = executor.submit(Callable {
            busyWait()
        })

        val incrementing: Future<Int> = executor.submit(Callable {
            var count = 0
            for (n in 1..100) {
                count = incrementIfBusy()
            }
            count
        })

        busyWaiting.get()
        incrementing.get()
    } finally {
        executor.shutdown()
    }
}

// busyWait will cycle for this many iterations.
// The faster your machine, the higher this needs to simulate busy waiting
// long enough for us to incrementIfBusy.
// If the Rust compiler gets too smart, and optimizes the busy wait loop away,
// then no number high enough will make the tests work.
val WAIT_FOR = 300 // ms

/// The first implementation uses Uniffi's default locking strategy.
/// Unfortunately, this counter does not work as we would like: uniffi
/// holds a mutex throughout the busyWait, and incrementIfBusy is only
/// called once that mutex is cleared, when it's not busy anymore.
Counter().use { counter -> 
    val count = countWhileBusy(
        { counter.busyWait(WAIT_FOR) },
        { counter.incrementIfBusy() }
    )
    assert(count == 0) { "Uniffi doing the locking: incrementIfBusy=$count" }
}

/// The second implementation is marked `[Threadsafe]` in the UDL,
/// which declares that the Rust library code will look after locking itself,
/// and not rely on the default locking strategy.
ThreadsafeCounter().use { counter -> 
    val count = countWhileBusy(
        { counter.busyWait(WAIT_FOR) },
        { counter.incrementIfBusy() }
    )
    assert(count > 0) { "Counter doing the locking: incrementIfBusy=$count" }
}