uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_futures.py",
    // Disable due to the Docker image being outdateed for now.
    // Please see https://github.com/mozilla/uniffi-rs/pull/1409#issuecomment-1437170423
    //
    // "tests/bindings/test_futures.swift",
    // "tests/bindings/test_futures.kts",
);
