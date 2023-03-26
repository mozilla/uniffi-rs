import fixture_version_mismatch;

// The thing we're testing is if loading the FFI library results in an error.
// Execute each UniFFI function to ensure the library is loaded.
let _ = aProcMacroExport()
let _ = aUdlFunction()

print("Script completed successfully")
