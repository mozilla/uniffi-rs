import uniffi.uniffi_bindgen_tests.*

val testInterface = TestInterface(20u)
assert(testInterface.getValue() == 20u)
assert(cloneInterface(testInterface).getValue() == 20u)

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
