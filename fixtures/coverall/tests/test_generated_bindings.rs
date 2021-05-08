uniffi_macros::build_foreign_language_testcases!(
    "src/coverall.udl",
    [
        "tests/bindings/test_coverall.py",
        "tests/bindings/test_coverall.kts",
        "tests/bindings/test_coverall.rb"
    ]
);
