uniffi_macros::build_foreign_language_testcases!(
    "src/arithmetic.idl",
    [
        "tests/bindings/test_arithmetic.py",
        "tests/bindings/test_arithmetic.kts",
        "tests/bindings/test_arithmetic.swift",
    ]
);
