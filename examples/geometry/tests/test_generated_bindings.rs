uniffi_macros::build_foreign_language_testcases!(
    ["src/geometry.udl",],
    [
        "tests/bindings/test_geometry.py",
        "tests/bindings/test_geometry.rb",
    ]
);
