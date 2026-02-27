/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

var callbackRefCount = 0;

final class Callback: TestCallbackInterface, @unchecked Sendable {
    var value: UInt32;

    init(value: UInt32) {
        self.value = value;
        callbackRefCount += 1
    }

    deinit {
        callbackRefCount -= 1
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
}

// Construct a callback interface to pass to rust
let cbi = Callback(value: 42);
// Test calling callback interface methods, which we can only do indirectly.
// Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
invokeTestCallbackInterfaceNoop(cbi: cbi);
assert(invokeTestCallbackInterfaceGetValue(cbi: cbi) == 42);
invokeTestCallbackInterfaceSetValue(cbi: cbi, value: 43);
assert(invokeTestCallbackInterfaceGetValue(cbi: cbi) == 43);

// The previcalls created a bunch of callback interface references.  Make sure they've been cleaned
// up and the only remaining reference is for our `cbi` variable.
assert(callbackRefCount == 1)
