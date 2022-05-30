uniffi_macros::build_foreign_language_testcases!(
    ["src/simple-fns.udl"],
    [
        "tests/bindings/test_simple_fns.kts",
        "tests/bindings/test_simple_fns.swift",
        "tests/bindings/test_simple_fns.py",
    ]
);
