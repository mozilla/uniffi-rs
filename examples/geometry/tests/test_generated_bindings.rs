uniffi_macros::build_foreign_language_testcases!(
    "src/geometry.idl",
    [
        "tests/bindings/test_geometry.py",
        "tests/bindings/test_geometry.kts",
        "tests/bindings/test_geometry.swift",
    ]
);
