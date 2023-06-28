use uniffi_nord as uniffi;

uniffi::build_foreign_language_testcases!(
    "tests/bindings/test_coverall_upstream_compatibility.py",
    "tests/bindings/test_coverall_upstream_compatibility.kts",
    "tests/bindings/test_coverall_upstream_compatibility.rb",
    "tests/bindings/test_coverall_upstream_compatibility.swift",
    "tests/bindings/test_handlerace_upstream_compatibility.kts",
);
