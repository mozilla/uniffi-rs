/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Tests for the callables listed in `bindings.swift.async_cancellable`.  Cancelling the Swift task
// that awaits one of them cancels and drops the Rust future, and the call throws
// `CancellationError`.

import Foundation
import uniffi_bindgen_tests

// Wait for a call that should have been cancelled and check that it threw `CancellationError`.
func expectCancellation<T: Sendable>(_ task: Task<T, Error>, _ what: String) async {
    do {
        let value = try await task.value
        fatalError("\(what): expected CancellationError, got \(value)")
    } catch is CancellationError {
    } catch {
        fatalError("\(what): expected CancellationError, got \(error)")
    }
}

// Runs on the main actor, so it keeps working while a Rust poll blocks a thread of the cooperative
// pool.
@MainActor
func waitUntil(_ what: String, _ condition: @Sendable () -> Bool) async {
    let deadline = Date().addingTimeInterval(5)
    while !condition() {
        if Date() > deadline {
            fatalError("timed out waiting for \(what)")
        }
        try! await Task.sleep(nanoseconds: 1_000_000)
    }
}

// Cancelled before the first poll: the task is already cancelled when the call starts, so the
// cancellation handler runs before Swift polls the Rust future.  Rust never polls the inner future.
do {
    let tester = CancellationTester(blockFirstPoll: false, completeAfterMs: nil)
    let task = Task.detached { () async throws -> Bool in
        withUnsafeCurrentTask { $0!.cancel() }
        return try await tester.run()
    }
    await expectCancellation(task, "cancelled before first poll")
    assert(tester.pollCount() == 0)
    assert(tester.liveCount() == 0)
}

// Cancelled while pending: Rust is waiting for a wakeup that never comes, so only the cancellation
// can end the call.
do {
    let tester = CancellationTester(blockFirstPoll: false, completeAfterMs: nil)
    let task = Task.detached { try await tester.run() }
    await waitUntil("first poll") { tester.liveCount() == 1 }
    task.cancel()
    await expectCancellation(task, "cancelled while pending")
    assert(tester.liveCount() == 0)
}

// Cancelled during a poll: `Task.cancel()` runs the cancellation handler synchronously, so
// `rust_future_cancel` runs while Rust is still inside the first poll.  When that poll returns
// `Pending`, Rust sees the cancellation and finishes the call without polling again.
do {
    let tester = CancellationTester(blockFirstPoll: true, completeAfterMs: nil)
    let task = Task.detached { try await tester.run() }
    await waitUntil("poll in progress") { tester.isInPoll() }
    task.cancel()
    assert(tester.isInPoll())
    tester.releasePoll()
    await expectCancellation(task, "cancelled during poll")
    assert(tester.pollCount() == 1)
    assert(tester.liveCount() == 0)
}

// Cancelled repeatedly, including after the call finished.  Swift runs the cancellation handler at
// most once and `rust_future_cancel` is idempotent anyway.
do {
    let tester = CancellationTester(blockFirstPoll: false, completeAfterMs: nil)
    let task = Task.detached { try await tester.run() }
    await waitUntil("first poll") { tester.liveCount() == 1 }
    task.cancel()
    task.cancel()
    await expectCancellation(task, "cancelled repeatedly")
    task.cancel()
    assert(tester.liveCount() == 0)
}

// Not cancelled: the call returns its value as usual.
do {
    let tester = CancellationTester(blockFirstPoll: false, completeAfterMs: 1)
    let completed = try await tester.run()
    assert(completed)
    assert(tester.liveCount() == 0)
}

// Cancellation racing completion and `rust_future_free`: cancel calls that complete around the time
// they're cancelled.  Each call must either return its value or throw `CancellationError`, and
// every Rust future must be dropped.
do {
    let testers = (0..<3).map { CancellationTester(blockFirstPoll: false, completeAfterMs: UInt32($0)) }
    var tasks: [Task<Bool, Error>] = []
    for i in 0..<300 {
        let tester = testers[i % testers.count]
        tasks.append(Task.detached { try await tester.run() })
    }
    for (i, task) in tasks.enumerated() {
        if i % 10 == 0 {
            try! await Task.sleep(nanoseconds: 200_000)
        }
        task.cancel()
    }
    for task in tasks {
        do {
            let completed = try await task.value
            assert(completed)
        } catch is CancellationError {
        } catch {
            fatalError("cancellation race: unexpected error \(error)")
        }
    }
    for tester in testers {
        assert(tester.liveCount() == 0)
    }
}
