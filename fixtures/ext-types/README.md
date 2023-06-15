This directory contains the tests for external types -- types defined in one crate and used in a
different one.

- `guid` and `uniffi-one` are dependent crates that define types exported by UniFFI
- `lib` is a library crate that depends on `guid` and `uniffi-one`
- `proc-macro-lib` is another library crate, but this one uses proc-macros rather than UDL files
