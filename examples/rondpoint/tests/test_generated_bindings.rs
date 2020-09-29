uniffi_macros::build_foreign_language_testcases!(
    "src/rondpoint.idl",
    [
        "tests/bindings/test_rondpoint.kts",
        "tests/bindings/test_rondpoint.swift",
        "tests/bindings/test_rondpoint.py",
    ]
);
