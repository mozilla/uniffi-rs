uniffi_macros::build_foreign_language_testcases!(
    ["src/simple-iface.udl"],
    [
        "tests/bindings/test_simple_iface.kts",
        "tests/bindings/test_simple_iface.swift",
        "tests/bindings/test_simple_iface.py",
    ]
);
