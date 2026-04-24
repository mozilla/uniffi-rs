import uniffi.uniffi_bindgen_tests.*

// the test here is just that we can successfully call a function across the FFI
testFunc()

try {
    testUnexpectedErrorFunc()
    throw RuntimeException("Expected uniffi.InternalException")
} catch (e: uniffi.InternalException) {
    assert(e.toString().contains("test panic"))
    // Expected
}
