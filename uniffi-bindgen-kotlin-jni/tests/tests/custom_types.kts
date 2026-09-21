import uniffi.uniffi_bindgen_tests.*

assert(roundtripCustomType1(100uL) == 100uL)
assert(roundtripCustomType2(mapOf("value" to 200uL)) == mapOf("value" to 200uL))

val i = CustomTypeInterface(67uL)
assert(roundtripCustomType3(i).getValue() == 67uL)
