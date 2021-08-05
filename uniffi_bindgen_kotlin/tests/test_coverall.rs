uniffi_macros::build_backend_language_testcases!(
    "../fixtures/coverall/src/coverall.udl", // TODO: how do we get the UDL from a published crate
    "../target/debug/deps/",                 // Is this OK?
    ["tests/test_coverall.kts", "tests/test_handlerace.kts"]
);

// An alternative to running the above macro (or a variation to help support the seperate backends)
// is to have the `coverall` have a runtime function that returns its udl
// or, alternatively, see if we can use some type of cargo metadata on runtime to retrieve
// the udl without having to download it from somewhere
