uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_futures.py",
    "tests/bindings/test_futures.swift",
    // Disable Kotlin test suite until the Docker container image is updated.
    // "tests/bindings/test_futures.kts",
);
