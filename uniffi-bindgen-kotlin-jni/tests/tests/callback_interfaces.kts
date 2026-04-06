import uniffi.uniffi_bindgen_tests.*

class CallbackImpl(var value: UInt) : TestCallbackInterface {
    override fun noop() {
    }

    override fun getValue(): UInt {
        return this.value
    }

    override fun setValue(value: UInt) {
        this.value = value
    }

    override fun throwIfEqual(numbers: CallbackInterfaceNumbers): CallbackInterfaceNumbers {
        if (numbers.a == numbers.b) {
            throw TestException.Failure1()
        } else {
            return numbers
        }
    }
}

// Construct a callback interface to pass to rust
val cbi = CallbackImpl(42u)
// Test calling callback interface methods, which we can only do indirectly.
// Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
invokeTestCallbackInterfaceNoop(cbi)
assert(invokeTestCallbackInterfaceGetValue(cbi) == 42u)
invokeTestCallbackInterfaceSetValue(cbi, 43u)
assert(invokeTestCallbackInterfaceGetValue(cbi) == 43u)

assert(invokeTestCallbackInterfaceThrowIfEqual(cbi, CallbackInterfaceNumbers(1u, 2u)) == CallbackInterfaceNumbers(1u, 2u))
try {
    invokeTestCallbackInterfaceThrowIfEqual(cbi, CallbackInterfaceNumbers(1u, 1u))
    throw RuntimeException("Expected TestException.Failure1 to be thrown")
} catch(e: TestException.Failure1) {
    // Expected
}
