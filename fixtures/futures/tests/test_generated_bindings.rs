uniffi_macros::build_foreign_language_testcases!(
    ["src/uniffi_futures.udl",],
    [
        "tests/bindings/test_futures.py",
        "tests/bindings/test_futures.swift",
        "tests/bindings/test_futures.kts",
    ]
);
