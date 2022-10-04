uniffi_macros::build_foreign_language_testcases!(
    ["src/proc-macro.udl"],
    [
        "tests/bindings/test_proc_macro.kts",
        "tests/bindings/test_proc_macro.swift",
        "tests/bindings/test_proc_macro.py",
    ]
);
