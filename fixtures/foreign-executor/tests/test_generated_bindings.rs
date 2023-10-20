uniffi::build_foreign_language_testcases!(
    // NordSecurity: Disabled because of intermittent CI failures
    // "tests/bindings/test_foreign_executor.py",
    "tests/bindings/test_foreign_executor.kts",
    // Disabled because of intermittent CI failures
    // (https://github.com/mozilla/uniffi-rs/issues/1536)
    // "tests/bindings/test_foreign_executor.swift",
);
