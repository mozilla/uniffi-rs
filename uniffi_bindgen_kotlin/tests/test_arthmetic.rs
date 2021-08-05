uniffi_macros::build_backend_language_testcases!(
    "../examples/arithmetic/src/arithmetic.udl",
    "../target/debug/deps",
    ["tests/test_arithmetic.kts",]
);
