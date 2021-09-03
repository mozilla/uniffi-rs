uniffi_macros::build_foreign_language_testcases!(
    ["src/callbacks.udl"],
    [
        "tests/bindings/test_callbacks.kts",
        "tests/bindings/test_callbacks.swift",
        //"tests/bindings/test_callbacks.py", // see https://github.com/mozilla/uniffi-rs/pull/1068
    ]
);
