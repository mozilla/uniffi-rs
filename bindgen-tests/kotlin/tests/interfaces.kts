import uniffi.uniffi_bindgen_tests.*

val testInterface = TestInterface(20u)
assert(testInterface.getValue() == 20u)
assert(cloneInterface(testInterface).getValue() == 20u)

// Test optional interfaces
assert(roundtripOptionalInterface(null) == null)
assert(roundtripOptionalInterface(TestInterface(20u))?.getValue() == 20u)

// Test secondary constructors
assert(TestInterface.secondaryConstructor(20u).getValue() == 40u)

// Test records that store interfaces
val two = TwoTestInterfaces(first=TestInterface(1u), second=TestInterface(2u))
val swapped = swapTestInterfaces(two)
assert(swapped.first.getValue() == 2u)
assert(swapped.second.getValue() == 1u)

// Test enums that store interfaces
val en = TestInterfaceEnum.One(TestInterface(1u))
assert(en.i.getValue() == 1u)

// Test argument name mapping
// the following calls will fail if the argument name differs
testInterface.methodWithMultiWordArg(theArgument="test")

// Other bindgens test that we free references, but that's much harder for Kotlin.
// For now, we don't test this at all

// Using an object after `close()` throws, both as a receiver and as an argument
val closedInterface = TestInterface(1u)
closedInterface.close()
try {
    closedInterface.getValue()
    throw RuntimeException("Expected IllegalStateException to be thrown")
} catch (e: IllegalStateException) {
    // Expected
}
try {
    cloneInterface(closedInterface)
    throw RuntimeException("Expected IllegalStateException to be thrown")
} catch (e: IllegalStateException) {
    // Expected
}
try {
    swapTestInterfaces(TwoTestInterfaces(first=closedInterface, second=TestInterface(2u)))
    throw RuntimeException("Expected IllegalStateException to be thrown")
} catch (e: IllegalStateException) {
    // Expected
}
