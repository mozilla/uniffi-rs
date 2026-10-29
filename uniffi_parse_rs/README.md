# `uniffi_parse_rs`

This crate parses Rust sources into UniFFI metadata that can then be used to generate the bindings.
This crate is experimental and not supported by any existing bindings.

# How to use it

Specify `src::[rust-crate-name]` as the source when generating bindings.
`uniffi_bindgen` will use `uniffi_parse_rs` to parse the source.

## Skipping parsing for some crates

Use the `package.metadata.uniffi` table in `Cargo.toml` to control which crates get parsed.

To skip a crate from being parsed ` use:
```
[package.metadata.uniffi]
skip_parsing = true
```

To skip a crate's dependencies from being parsed use:
```
[package.metadata.uniffi]
skip_dependencies = [
    "crate-one",
    "crate-two",
    // ...
]
```

# What works

* `#[derive(uniffi::Record)]`, `#[derive(uniffi::Enum)]` and `#[derive(uniffi::Object)]`
* `#[uniffi::export]`
* `uniffi::custom_type!`
* Recursing through modules -- both inline and in separate source files.
* Parsing UDL and combining the metadata.
* Conditional compilation based on features and the `#[cfg]` and `#[cfg_attr]` macros
    * The attributes must be spelled out as `cfg` and `cfg_attr`.
      Renaming them via a `use` statement is not supported,
      nor is renaming other macros to `cfg` or `cfg_attr`.
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
