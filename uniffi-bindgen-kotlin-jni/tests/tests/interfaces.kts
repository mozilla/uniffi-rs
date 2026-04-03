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

// Test that references are freed

// Create 2 references to the same interface
val refCountInterface = TestInterface(1u);
var refCountInterface2: TestInterface? = cloneInterface(refCountInterface)

// Call a bunch of functions that create other temporary references
refCountInterface.getValue()
cloneInterface(refCountInterface)


// Clear the second reference, wait for the GC to run, then check that all references have been
// freed, except for the one corresponding to the `refCountInterface` variable.
refCountInterface2 = null
for (i in 1..100) {
    System.gc()
    Thread.sleep(100)
    if (refCountInterface.refCount() == 1u) {
        break
    }
}
assert(refCountInterface.refCount() == 1u)
