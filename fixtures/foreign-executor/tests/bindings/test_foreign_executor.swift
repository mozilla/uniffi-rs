/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import fixture_foreign_executor

func runTest(tester: ForeignExecutorTester, delay: UInt32) async -> TestResult {
    let handle = Task { () -> TestResult in
        tester.scheduleTest(delay: delay)
        try! await Task.sleep(nanoseconds: numericCast((delay + 10) * 1000000))
        return tester.getLastResult()!
    }
    return await handle.value
}

Task {
    // Test scheduling with no delay
    let result = await runTest(
        tester: ForeignExecutorTester(
            executor: UniFfiForeignExecutor(priority: TaskPriority.background)
        ),
        delay: 0
    )
    assert(result.callHappenedInDifferentThread)
    assert(result.delayMs <= 1)

    // Test scheduling with delay and an executor created from a list
    let result2 = await runTest(
        tester: ForeignExecutorTester.newFromSequence(
            executors: [UniFfiForeignExecutor(priority: TaskPriority.background)]
        ),
        delay: 1000
    )
    assert(result2.callHappenedInDifferentThread)
    assert(result2.delayMs >= 90)
    assert(result2.delayMs <= 110)
}



// No need to test reference counting, since `UniFfiForeignExecutor` on Swift is just a value type
