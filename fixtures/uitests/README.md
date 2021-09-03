# A suite of "user interface" tests for UniFFI.

This crate uses [trybuild](https://docs.rs/trybuild) to automate tests for the compiler
output of UniFFI-generated code. This helps us ensure that we're generatuing useful
error messages in cases where the user's Rust code and `.udl` file to not match
up correctly.

Ideally these tests would be part of the `uniffi_bindgen` crate, but factoring it out
into a separate crate has made it easier to integrate with `trybuild`. In particular
it lets us use convenience macros from `uniffi_macros` when writing the tests, without
having to deal with a circular dependency between `uniffi_macros` and `uniffi_bindgen`.