uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_futures.py",
    "tests/bindings/test_futures.swift",
    // Disable due to the Docker image being outdated for now
    // Please see https://github.com/mozilla/uniffi-rs/pull/1409#issuecomment-1433871031.
    // "tests/bindings/test_futures.kts",
);
