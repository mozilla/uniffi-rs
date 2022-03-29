uniffi_macros::build_foreign_language_testcases!(
    ["src/external-types-lib.udl",],
    [
        "tests/bindings/test_external_types.py",
        "tests/bindings/test_external_types.rb",
        "tests/bindings/test_external_types.swift",
    ]
);
