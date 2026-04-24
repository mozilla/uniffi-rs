import uniffi.uniffi_bindgen_tests.*

var bytes = ByteArray(4)
bytes[0] = 0.toByte()
bytes[1] = 1.toByte()
bytes[2] = 2.toByte()
bytes[3] = 3.toByte()
assert(roundtripBytes(bytes).contentEquals(bytes))
