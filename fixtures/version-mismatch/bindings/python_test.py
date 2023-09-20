from fixture_version_mismatch import *

# The thing we're testing is if loading the FFI library results in an error.
# Execute each UniFFI function to ensure the library is loaded.
a_proc_macro_export()
a_udl_function()

print("Script completed successfully")
