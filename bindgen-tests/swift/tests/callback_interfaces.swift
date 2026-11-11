/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import uniffi_bindgen_tests

// We want to mutate callbackRefCount from async swift code.
// The simplest way to do that seems to be to mark it `nonisolated(unsafe)`.
nonisolated(unsafe) var callbackRefCount = 0;
// Let's use this lock whenever we mutate callbackRefCount, just to make sure that everything is
// thread-safe.
nonisolated(unsafe) var callbackRefCountLock = NSLock()

final class Callback: TestCallbackInterface, @unchecked Sendable {
    var value: UInt32;

    init(value: UInt32) {
        self.value = value;
        callbackRefCountLock.withLock {
            callbackRefCount += 1
        }
    }

    deinit {
        callbackRefCountLock.withLock {
            callbackRefCount -= 1
        }
    }

    func noop() {}

    func getValue() -> UInt32 {
        return self.value;
    }

    func setValue(value: UInt32) {
        self.value = value;
    }

    func throwIfEqual(numbers: CallbackInterfaceNumbers) throws  -> CallbackInterfaceNumbers {
        if numbers.a == numbers.b {
            throw TestError.Failure1
        }
        return numbers
    }

    func echo(s: String) -> String {
        return s
    }
}

// Construct a callback interface to pass to rust
let cbi = Callback(value: 42);
// Test calling callback interface methods, which we can only do indirectly.
// Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
invokeTestCallbackInterfaceNoop(cbi: cbi);
assert(invokeTestCallbackInterfaceGetValue(cbi: cbi) == 42);
invokeTestCallbackInterfaceSetValue(cbi: cbi, value: 43);
assert(invokeTestCallbackInterfaceGetValue(cbi: cbi) == 43);
assert(invokeTestCallbackInterfaceEcho(cbi: cbi, s: "test-string") == "test-string");

assert(invokeOptionalTestCallbackInterfaceGetValue(cbi: nil) == nil);
assert(invokeOptionalTestCallbackInterfaceGetValue(cbi: cbi) == 43);

// The previcalls created a bunch of callback interface references.  Make sure they've been cleaned
// up and the only remaining reference is for our `cbi` variable.
callbackRefCountLock.withLock {
    assert(callbackRefCount == 1)
}

// Calls that re-enter Rust from inside a callback, so several calls are in progress on the same
// thread at once. An inner call's failure must not become the result of the call it's nested in,
// and the outer call's own result must still come through after the inner calls.
final class ReentrantCallback: TestReentrantCallback, @unchecked Sendable {
    func reenter(depth: UInt32) throws -> UInt32 {
        // A nested call that fails, caught here. The call this is nested in must still succeed.
        do {
            try funcWithError(input: 0)
            fatalError("Expected TestError.Failure1 to be thrown")
        } catch TestError.Failure1 {
            // Expected
        }
        if depth == 0 {
            return 0
        }
        return try invokeTestReentrantCallback(cbi: self, depth: depth - 1, fail: false)
    }
}

let reentrant = ReentrantCallback()
assert(try! invokeTestReentrantCallback(cbi: reentrant, depth: 0, fail: false) == 1)
// 20 levels of Rust -> Swift -> Rust nesting, each with a failed call nested inside it.
assert(try! invokeTestReentrantCallback(cbi: reentrant, depth: 20, fail: false) == 21)
// The outer call fails after the nested calls (including the failed ones) have finished, and
// must report its own error.
do {
    _ = try invokeTestReentrantCallback(cbi: reentrant, depth: 5, fail: true)
    fatalError("Expected TestError.Failure3 to be thrown")
} catch TestError.Failure3(let code) {
    assert(code == 5)
}
// The same thread keeps working after the failures above.
assert(try! invokeTestReentrantCallback(cbi: reentrant, depth: 3, fail: false) == 4)

// Several threads making reentrant calls, failing and succeeding, at the same time.
DispatchQueue.concurrentPerform(iterations: 8) { t in
    for i in 0..<200 {
        let depth = UInt32((t + i) % 4)
        do {
            if i % 3 == 0 {
                do {
                    _ = try invokeTestReentrantCallback(cbi: reentrant, depth: depth, fail: true)
                    fatalError("Expected TestError.Failure3 to be thrown")
                } catch TestError.Failure3(let code) {
                    assert(code == depth)
                }
            } else {
                let value = try invokeTestReentrantCallback(cbi: reentrant, depth: depth, fail: false)
                assert(value == depth + 1)
            }
        } catch {
            fatalError("Unexpected error from a reentrant call: \(error)")
        }
    }
}
