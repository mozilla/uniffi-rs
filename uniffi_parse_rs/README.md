# `uniffi_parse_rs`

This crate parses Rust sources into UniFFI metadata that can then be used to generate the bindings.
This crate is experimental and not supported by any existing bindings.

# What works

* `#[derive(uniffi::Record)]`, `#[derive(uniffi::Enum)]` and `#[derive(uniffi::Object)]`
* `#[uniffi::export]`
* `uniffi::custom_type!`
* Recursing through modules -- both inline and in separate source files.
* Parsing UDL and combining the metadata.
* Conditional compilation based on features and the `#[cfg]` and `#[cfg_attr]` macros
* External types in the parsed crates, i.e. following `use` statements to find the source type
* Type aliases in the parsed crates

# What doesn't work

* macros or build scripts (other than the specific macros listed above).
* Parsing inside function bodies.  All items must be top-level items in their modules.
* `use` statements or Type aliases that go through crates that aren't parsed.

# Extra requirements
* Types and functions used in the API must be `pub`.
* Types and functions used in the API must be exported at the crate root.
  (This requirement could be removed, but it requires new macro syntax).
