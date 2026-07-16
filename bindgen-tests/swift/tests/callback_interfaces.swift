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

// The previcalls created a bunch of callback interface references.  Make sure they've been cleaned
// up and the only remaining reference is for our `cbi` variable.
callbackRefCountLock.withLock {
    assert(callbackRefCount == 1)
}
