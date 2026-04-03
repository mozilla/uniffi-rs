import uniffi.uniffi_bindgen_tests.*

assert(roundtripCustomType1(100uL) == 100uL)
assert(roundtripCustomType2(mapOf("value" to 200uL)) == mapOf("value" to 200uL))
