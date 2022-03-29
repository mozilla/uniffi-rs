uniffi_macros::build_foreign_language_testcases!(
    ["src/chronological.udl",],
    [
        "tests/bindings/test_chronological.py",
        "tests/bindings/test_chronological.swift",
    ]
);
