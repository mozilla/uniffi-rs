uniffi_macros::build_foreign_language_testcases!(
    ["src/todolist.udl",],
    [
        "tests/bindings/test_todolist.rb",
        "tests/bindings/test_todolist.py"
    ]
);
