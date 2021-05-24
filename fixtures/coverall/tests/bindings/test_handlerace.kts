/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*
import uniffi.coverall.*

// This tests the thread-safety of operations on object handles.
// The idea is to have a method call running on one thread while
// we try to destroy the object on another thread. This should
// be handled correctly at the Kotlin level, rather than triggering
// any fail-safe checks (or worse, memory unsafety!) in the underlying
// Rust code.
//
// This test was able to reproduce the concurrency issue in
// https://github.com/mozilla/uniffi-rs/issues/457, and is kept
// to prevent regressions.

// Give ourselves enough opportunity to trigger concurrency issues.
for (i in 1..1000) {
    // So we can operate concurrently.
    val executor = Executors.newFixedThreadPool(2)
    try {
        // On the main thread here, we're going to create and then destroy a `Coveralls` instance.
        val concurrentMethodCall: Future<String> = Coveralls("test_handlerace").use { coveralls ->
            // Make a method call on a separate thread thread.
            val concurrentMethodCall = executor.submit(Callable {
                coveralls.getName()
            })
            // Sleep a little, to give the background thread a chance to start operating.
            Thread.sleep(1)
            concurrentMethodCall
        }
        // At this point the object has been destroyed.
        // The concurrent method call should either have failed to run at all (if the object was destroyed
        // before it started) or should have completely succeeded (if it started before the object was
        // destroyed). It should not fail in the Rust code (which currently fails cleanly on handlemap
        // safety checks, but in future might be a use-after-free).
        try {
            concurrentMethodCall.get()
        } catch (e: ExecutionException) {
            if (e.cause is IllegalStateException && e.message!!.endsWith("has already been destroyed")) {
                // This is fine, we rejected the call after the object had been destroyed.
            } else {
                // Any other behaiour signals a possible problem.
                throw e
            }
        }
    } finally {
        executor.shutdown()
    }
}
