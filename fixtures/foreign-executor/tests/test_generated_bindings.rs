uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_foreign_executor.py",
    // Disable due to the Docker image being outdated for now.
    // Please see https://github.com/mozilla/uniffi-rs/pull/1409#issuecomment-1437170423
    //
    // "tests/bindings/test_foreign_executor.kts",
    // "tests/bindings/test_foreign_executor.swift",
);
