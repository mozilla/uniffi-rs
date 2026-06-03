import uniffi.uniffi_bindgen_tests.*


try {
    funcWithError(0u)
    throw RuntimeException("Should have thrown TestException.Failure1")
} catch (e: TestException.Failure1) {
    // Expected
}

try {
    funcWithError(1u)
    throw RuntimeException("Should have thrown TestException.Failure2")
} catch (e: TestException.Failure2) {
    // Expected
    assert(e.data == "DATA")
}

try {
    funcWithError(50u)
    throw RuntimeException("Should have thrown TestException.Failure3")
} catch (e: TestException.Failure3) {
    assert(e.v1 == 50u)
}

try {
    funcWithFlatError(0u)
    throw RuntimeException("Should have thrown TestException.Failure1")
} catch (e: TestFlatException.IoException) {
    // Expected
}

// These shouldn't throw
funcWithError(200u)
funcWithFlatError(1u)

// Error enum with no data
try {
    funcWithErrorNoData(0u)
    throw RuntimeException("Should have thrown TestErrorNoData.Failure1")
} catch (e: TestErrorNoData.Failure1) {
    // Expected
}

try {
    funcWithErrorNoData(1u)
    throw RuntimeException("Should have thrown TestErrorNoData.Failure2")
} catch (e: TestErrorNoData.Failure2) {
    // Expected
}

try {
    funcWithErrorNoData(2u)
    throw RuntimeException("Should have thrown TestErrorNoData.Failure3")
} catch (e: TestErrorNoData.Failure3) {
    // Expected
}

// This shouldn't throw
funcWithErrorNoData(200u)
