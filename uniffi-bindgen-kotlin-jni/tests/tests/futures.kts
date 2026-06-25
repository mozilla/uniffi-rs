import kotlinx.coroutines.*
import uniffi.uniffi_bindgen_tests.*

// Simple functions
runBlocking {
    assert(asyncRoundtripU8(42.toUByte()) == 42.toUByte())
    assert(asyncRoundtripI8((-42).toByte()) == (-42).toByte())
    assert(asyncRoundtripU16(42.toUShort()) == 42.toUShort())
    assert(asyncRoundtripI16((-42).toShort()) == (-42).toShort())
    assert(asyncRoundtripU32(42u) == 42u)
    assert(asyncRoundtripI32(-42) == -42)
    assert(asyncRoundtripU64(42uL) == 42uL)
    assert(asyncRoundtripI64(-42L) == -42L)
    assert(asyncRoundtripF32(0.5f) == 0.5f)
    assert(asyncRoundtripF64(-0.5) == -0.5)
    assert(asyncRoundtripString("hi") == "hi")
    assert(asyncRoundtripVec(listOf(42u)) == listOf(42u))
    assert(asyncRoundtripMap(mapOf("hello" to "world")) == mapOf("hello" to "world"))
}

// Errors
runBlocking {
    try {
        asyncThrowError()
        throw RuntimeException("Expected TestException.Failure1")
    } catch(e: TestException.Failure1) {
        // expected
    }
}

// Objects and methods
runBlocking {
    val obj = AsyncInterface("Alice")
    assert(obj.name() == "Alice")

    val obj2 = asyncRoundtripObj(obj)
    assert(obj2.name() == "Alice")
}


// Async callback interfaces
class AsyncCallbackImpl(var value: UInt) : TestAsyncCallbackInterface {
    override suspend fun noop() { }

    override suspend fun getValue(): UInt {
        return this.value
    }

    override suspend fun setValue(value: UInt) {
        this.value = value
    }

    override suspend fun throwIfEqual(numbers: CallbackInterfaceNumbers): CallbackInterfaceNumbers {
        if (numbers.a == numbers.b) {
            throw TestException.Failure1()
        }
        return numbers
    }
}

runBlocking {
    val cbi = AsyncCallbackImpl(42u)
    invokeTestAsyncCallbackInterfaceNoop(cbi)

    assert(invokeTestAsyncCallbackInterfaceGetValue(cbi) == 42u)

    invokeTestAsyncCallbackInterfaceSetValue(cbi, 43u)
    assert(invokeTestAsyncCallbackInterfaceGetValue(cbi) == 43u)

    try {
       invokeTestAsyncCallbackInterfaceThrowIfEqual(
            cbi,
            CallbackInterfaceNumbers(a=10u, b=10u)
        )
        throw RuntimeException("Expected TestException.Failure1")
    } catch (e: TestException.Failure1) {
        // expected
    }

    assert(invokeTestAsyncCallbackInterfaceThrowIfEqual(
        cbi,
        CallbackInterfaceNumbers(a=10u, b=11u)
    ) == CallbackInterfaceNumbers(a=10u, b=11u))
}
