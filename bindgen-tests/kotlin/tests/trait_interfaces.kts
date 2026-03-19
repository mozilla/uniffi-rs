import kotlinx.coroutines.*
import uniffi.uniffi_bindgen_tests.*

class TraitImpl(var value: UInt) : TestTraitInterface {
    override fun noop() { }

    override fun getValue(): UInt {
        return this.value
    }

    override fun setValue(value: UInt) {
        this.value = value
    }

    override fun throwIfEqual(numbers: CallbackInterfaceNumbers): CallbackInterfaceNumbers {
        if (numbers.a == numbers.b) {
            throw TestException.Failure1()
        }
        return numbers
    }
}

fun checkRustImpl(rustTraitImpl: TestTraitInterface) {
    rustTraitImpl.noop()
    assert(rustTraitImpl.getValue() == 42u)
    rustTraitImpl.setValue(43u)
    assert(rustTraitImpl.getValue() == 43u)
    try {
        rustTraitImpl.throwIfEqual(CallbackInterfaceNumbers(a=10u, b=10u))
        throw RuntimeException("Expected TestException.Failure1")
    } catch (e: TestException.Failure1) {
        // Expected
    }
    assert(
        rustTraitImpl.throwIfEqual(CallbackInterfaceNumbers(a=10u, b=11u)) ==
        CallbackInterfaceNumbers(a=10u, b=11u)
    )
}

fun checkKtImpl(ktTraitImpl: TestTraitInterface) {
    invokeTestTraitInterfaceNoop(ktTraitImpl)
    assert(invokeTestTraitInterfaceGetValue(ktTraitImpl) == 42u)
    invokeTestTraitInterfaceSetValue(ktTraitImpl, 43u)
    assert(invokeTestTraitInterfaceGetValue(ktTraitImpl) == 43u)

    try {
        invokeTestTraitInterfaceThrowIfEqual(
            ktTraitImpl,
            CallbackInterfaceNumbers(a=10u, b=10u)
        )
        throw RuntimeException("Expected TestException.Failure1")
    } catch (e: TestException.Failure1) {
        // Expected
    }
    assert(
        invokeTestTraitInterfaceThrowIfEqual(
            ktTraitImpl,
            CallbackInterfaceNumbers(a=10u, b=11u)
        ) == CallbackInterfaceNumbers(a=10u, b=11u)
    )
}

checkRustImpl(createTestTraitInterface(42u))
checkRustImpl(
    roundtripTestTraitInterface(createTestTraitInterface(42u))
)
checkRustImpl(
    roundtripTestTraitInterfaceList(listOf(createTestTraitInterface(42u)))[0]
)

checkKtImpl(TraitImpl(42u))
checkKtImpl(roundtripTestTraitInterface(TraitImpl(42u)))
checkKtImpl(
    roundtripTestTraitInterfaceList(listOf(TraitImpl(42u)))[0]
)



class AsyncTraitImpl(var value: UInt) : AsyncTestTraitInterface {
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

suspend fun checkAsyncRustImpl(rustTraitImpl: AsyncTestTraitInterface) {
    rustTraitImpl.noop()
    assert(rustTraitImpl.getValue() == 42u)
    rustTraitImpl.setValue(43u)
    assert(rustTraitImpl.getValue() == 43u)
    try {
        rustTraitImpl.throwIfEqual(CallbackInterfaceNumbers(a=10u, b=10u))
        throw RuntimeException("Expected TestException.Failure1")
    } catch (e: TestException.Failure1) {
        // Expected
    }
    assert(
        rustTraitImpl.throwIfEqual(CallbackInterfaceNumbers(a=10u, b=11u)) ==
        CallbackInterfaceNumbers(a=10u, b=11u)
    )
}

suspend fun checkAsyncKtImpl(ktTraitImpl: AsyncTestTraitInterface) {
    invokeAsyncTestTraitInterfaceNoop(ktTraitImpl)
    assert(invokeAsyncTestTraitInterfaceGetValue(ktTraitImpl) == 42u)
    invokeAsyncTestTraitInterfaceSetValue(ktTraitImpl, 43u)
    assert(invokeAsyncTestTraitInterfaceGetValue(ktTraitImpl) == 43u)

    try {
        invokeAsyncTestTraitInterfaceThrowIfEqual(
            ktTraitImpl,
            CallbackInterfaceNumbers(a=10u, b=10u)
        )
        throw RuntimeException("Expected TestException.Failure1")
    } catch (e: TestException.Failure1) {
        // Expected
    }
    assert(
        invokeAsyncTestTraitInterfaceThrowIfEqual(
            ktTraitImpl,
            CallbackInterfaceNumbers(a=10u, b=11u)
        ) == CallbackInterfaceNumbers(a=10u, b=11u)
    )
}

runBlocking {
    checkAsyncRustImpl(createAsyncTestTraitInterface(42u))
    checkAsyncRustImpl(
        roundtripAsyncTestTraitInterface(createAsyncTestTraitInterface(42u))
    )
    checkAsyncRustImpl(
        roundtripAsyncTestTraitInterfaceList(listOf(createAsyncTestTraitInterface(42u)))[0]
    )
}

runBlocking {
    checkAsyncKtImpl(AsyncTraitImpl(42u))
    checkAsyncKtImpl(roundtripAsyncTestTraitInterface(AsyncTraitImpl(42u)))
    checkAsyncKtImpl(
        roundtripAsyncTestTraitInterfaceList(listOf(AsyncTraitImpl(42u)))[0]
    )
}
