uniffi_macros::build_foreign_language_testcases!(
    "tests/bindings/test_imported_types.swift",
    // No need to test other languages, since this crate only changes the Swift configuration
    // (see ../README.md)
);
