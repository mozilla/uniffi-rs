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
* Checksum checks are currently disabled (https://github.com/mozilla/uniffi-rs/issues/2932)

# Extra requirements
* Types and functions used in the API must be `pub` and reachable from outside crates.
* The types used in macros like `remote_type!` and `custom_type!`
  must be the same types used in exported functions.
  Previously we allowed one to be a type alias of the other, but this is no longer allowed.
