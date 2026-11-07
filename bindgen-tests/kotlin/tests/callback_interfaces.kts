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
        if (numbers.a == 6u && numbers.b == 7u) {
            throw UnexpectedFailure()
        } else if (numbers.a == numbers.b) {
            throw TestException.Failure1()
        } else {
            return numbers
        }
    }

    override fun echo(s: String): String {
        return s
    }
}

// Unexpected error that we'll throw.
//
// This inherits from Throwable, since we want to support errors that don't subclass Exception
// (https://github.com/mozilla/uniffi-rs/issues/2903)
class UnexpectedFailure : Throwable("unexpected failure")

// Construct a callback interface to pass to rust
val cbi = CallbackImpl(42u)
// Test calling callback interface methods, which we can only do indirectly.
// Each of these Rust functions inputs a callback interface, calls a method on it, then returns the result.
invokeTestCallbackInterfaceNoop(cbi)
assert(invokeTestCallbackInterfaceGetValue(cbi) == 42u)
invokeTestCallbackInterfaceSetValue(cbi, 43u)
assert(invokeTestCallbackInterfaceGetValue(cbi) == 43u)
assert(invokeTestCallbackInterfaceEcho(cbi, "test-string") == "test-string")


assert(invokeTestCallbackInterfaceThrowIfEqual(cbi, CallbackInterfaceNumbers(1u, 2u)) == CallbackInterfaceNumbers(1u, 2u))
try {
    invokeTestCallbackInterfaceThrowIfEqual(cbi, CallbackInterfaceNumbers(1u, 1u))
    throw RuntimeException("Expected TestException.Failure1 to be thrown")
} catch(e: TestException.Failure1) {
    // Expected
}

assert(invokeOptionalTestCallbackInterfaceGetValue(null) == null)
assert(invokeOptionalTestCallbackInterfaceGetValue(cbi) == 43u)

// Test unexpected errors
try {
    invokeTestCallbackInterfaceThrowIfEqual(cbi, CallbackInterfaceNumbers(6u, 7u))
    throw RuntimeException("Expected RuntimeException to be caught, converted to TestException.Failure2, and thrown")
} catch(e: TestException.Failure2) {
    assert(e.data.contains("unexpected failure"))
}

// Test calls that re-enter Rust from inside a callback, so several calls are in progress on the
// same thread at once. An inner call's failure must not become the result of the call it's
// nested in, and the outer call's own result must still come through after the inner calls.
class ReentrantCallback : TestReentrantCallback {
    override fun reenter(depth: UInt): UInt {
        // A nested call that fails, caught here. The call this is nested in must still succeed.
        try {
            funcWithError(0u)
            throw RuntimeException("Expected TestException.Failure1 to be thrown")
        } catch(e: TestException.Failure1) {
            // Expected
        }
        if (depth == 0u) {
            return 0u
        }
        return invokeTestReentrantCallback(this, depth - 1u, false)
    }
}

val reentrant = ReentrantCallback()
assert(invokeTestReentrantCallback(reentrant, 0u, false) == 1u)
// 20 levels of Rust -> Kotlin -> Rust nesting, each with a failed call nested inside it.
assert(invokeTestReentrantCallback(reentrant, 20u, false) == 21u)
// The outer call fails after the nested calls (including the failed ones) have finished, and
// must report its own error.
try {
    invokeTestReentrantCallback(reentrant, 5u, true)
    throw RuntimeException("Expected TestException.Failure3 to be thrown")
} catch(e: TestException.Failure3) {
    assert(e.v1 == 5u)
}
// The same thread keeps working after the failures above.
assert(invokeTestReentrantCallback(reentrant, 3u, false) == 4u)

// Several threads making reentrant calls, failing and succeeding, at the same time.
fun reentrantCalls(t: Int) {
    for (i in 0 until 200) {
        val depth = ((t + i) % 4).toUInt()
        if (i % 3 == 0) {
            try {
                invokeTestReentrantCallback(reentrant, depth, true)
                throw RuntimeException("Expected TestException.Failure3 to be thrown")
            } catch(e: TestException.Failure3) {
                assert(e.v1 == depth)
            }
        } else {
            assert(invokeTestReentrantCallback(reentrant, depth, false) == depth + 1u)
        }
    }
}

// Exceptions thrown on those threads are collected and rethrown here.
val reentrantFailures = java.util.concurrent.ConcurrentLinkedQueue<Throwable>()
val reentrantThreads = (0 until 8).map { t ->
    kotlin.concurrent.thread {
        try {
            reentrantCalls(t)
        } catch(e: Throwable) {
            reentrantFailures.add(e)
        }
    }
}
reentrantThreads.forEach { it.join() }
reentrantFailures.peek()?.let { throw it }
