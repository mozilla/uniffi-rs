import uniffi.uniffi_bindgen_tests.*

assert(roundtripCustomType1(100uL) == 100uL)
assert(roundtripCustomType2(mapOf("value" to 200uL)) == mapOf("value" to 200uL))

val i = CustomTypeInterface(67uL)
assert(roundtripCustomType3(i).getValue() == 67uL)

assert(roundtripCustomType1Opt(100uL) == 100uL)
assert(roundtripCustomType2Opt(mapOf("value" to 200uL)) == mapOf("value" to 200uL))
assert(roundtripCustomType3Opt(i)?.getValue() == 67uL)

assert(roundtripCustomType1Opt(null) == null)
assert(roundtripCustomType2Opt(null) == null)
assert(roundtripCustomType3Opt(null) == null)

assert(tryRoundtripCustomType1(100uL) == 100uL)
try {
    tryRoundtripCustomType1(20000uL)
    throw RuntimeException("Should have thrown CustomTypeException.Overflow")
} catch (e: CustomTypeException.Overflow) {
    // Expected
}

