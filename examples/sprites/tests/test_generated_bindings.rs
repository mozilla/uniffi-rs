uniffi_macros::build_foreign_language_testcases!(
    "src/sprites.udl",
    [
        "tests/bindings/test_sprites.py",
        "tests/bindings/test_sprites.kts",
        "tests/bindings/test_sprites.swift",
    ]
);
