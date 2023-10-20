uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_futures.py",
    "tests/bindings/test_futures.swift",
    // Disabled, because async functions are flaky:
    // https://github.com/mozilla/uniffi-rs/pull/1677
    // "tests/bindings/test_futures.kts",
);
