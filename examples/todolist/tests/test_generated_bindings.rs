uniffi_macros::build_foreign_language_testcases!(
    "src/todolist.udl",
    [
        "tests/bindings/test_todolist.kts",
        "tests/bindings/test_todolist.swift",
        "tests/bindings/test_todolist.py"
    ]
);
