import uniffi.uniffi_bindgen_tests.*


try {
    funcWithError(0u)
    throw RuntimeException("Should have thrown TestException.Failure1")
} catch (e: TestException.Failure1) {
    // Expected
}

try {
    funcWithError(1u)
    throw RuntimeException("Should have thrown TestException.Failure1")
} catch (e: TestException.Failure2) {
    // Expected
}

try {
    funcWithFlatError(0u)
    throw RuntimeException("Should have thrown TestException.Failure1")
} catch (e: TestFlatException.IoException) {
    // Expected
}

// These shouldn't throw
funcWithError(2u)
funcWithFlatError(1u)
