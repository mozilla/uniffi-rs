import uniffi.uniffi_bindgen_tests.*

val testInterface = TestInterface(20u)
assert(testInterface.getValue() == 20u)
assert(cloneInterface(testInterface).getValue() == 20u)

val two = TwoTestInterfaces(first=TestInterface(1u), second=TestInterface(2u))
val swapped = swapTestInterfaces(two)
assert(swapped.first.getValue() == 2u)
assert(swapped.second.getValue() == 1u)

// Other bindgens test that we free references, but that's much harder for Kotlin.
// For now, we don't test this at all
