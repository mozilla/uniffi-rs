uniffi_macros::build_foreign_language_testcases!(
    ["src/wrapper-types.udl"],
    [
        "tests/bindings/test_wrapper_types.kts",
        "tests/bindings/test_wrapper_types.py",
        "tests/bindings/test_wrapper_types.swift",
    ]
);
