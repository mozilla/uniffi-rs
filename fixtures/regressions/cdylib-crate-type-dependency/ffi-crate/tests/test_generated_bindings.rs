uniffi_macros::build_foreign_language_testcases!(
    ["src/test.udl",],
    [
        "tests/bindings/test.py",
        "tests/bindings/test.swift",
    ]
);
