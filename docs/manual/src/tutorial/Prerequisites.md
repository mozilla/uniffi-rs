# Prerequisites

## Bindgen CLI tools

Install the `uniffi-bindgen` binary on your system using:

`cargo install uniffi_bindgen`

Install one or more binaries for the foreign languages you need bindings for.
  - `cargo install uniffi_bindgen_kotlin` for `uniffi-bindgen-kotlin`
  - `cargo install uniffi_bindgen_swift` for `uniffi-bindgen-swift`
  - `cargo install uniffi_bindgen_python` for `uniffi-bindgen-python`
  - `cargo install uniffi_bindgen_ruby` for `uniffi-bindgen-ruby`
  - Search crates.io for other bindings generators.

You can see what these can do with `uniffi-bindgen --help`, but let's leave it aside for now.

## Build your crate as a cdylib

Ensure your crate builds as a `cdylib` by adding
```toml
crate-type = ["cdylib"]
name = "<library name>"
```
to your crate's `Cargo.toml`.
