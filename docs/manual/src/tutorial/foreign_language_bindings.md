# Foreign-language bindings

As stated in the [Overview](../Overview.md), this library and tutorial does not cover *how* to ship a Rust library on mobile, but how to generate bindings for it, so this section will only cover that.

## Creating the bindgen binary

First, make sure you have installed all the [prerequisites](./Prerequisites.md).

Ideally you would then run the `uniffi-bindgen` binary from the `uniffi` crate to generate your bindings.  However, this
is only available with [Cargo nightly](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies).
To work around this, you need to create a binary in your project that does the same thing.

Add the following to your `Cargo.toml`:

```toml
[[bin]]
# This can be whatever name makes sense for your project, but the rest of this tutorial assumes uniffi-bindgen.
name = "uniffi-bindgen"
path = "uniffi-bindgen.rs"
```

Create `uniffi-bindgen.rs`:
```rust
fn main() {
    uniffi::uniffi_bindgen_main()
}
```

You can now run `uniffi-bindgen` from your project using `cargo run --features=uniffi/cli --bin uniffi-bindgen [args]`

### Multi-crate workspaces

If your project consists of multiple crates in a Cargo workspace, then the process outlined above would require you
creating a binary for each crate that uses UniFFI.  You can avoid this by creating a separate crate for running `uniffi-bindgen`:
  - Name the crate `uniffi-bindgen`
  - Add this dependency to `Cargo.toml`: `uniffi = {version = "0.XX.0", features = ["cli"] }`
  - Follow the steps from the previous section to add the `uniffi-bindgen` binary target

Then your can run `uniffi-bindgen` from any create in your project using `cargo run -p uniffi-bindgen [args]`

## Running uniffi-bindgen using a library file (RECOMMENDED)

Use `generate --library` to generate foreign bindings by using a cdylib file built for your library.
This flag was added in UniFFI 0.24 and can be more convenient than specifying the UDL file -- especially when multiple UniFFI-ed crates are built together in one library.
The plan is to make library mode the default in a future UniFFI version, and it is highly recommended to specify the flag for now.

Taking `example/arithmetic` as an example, you can generate the bindings with:
```
cargo build --release
cargo run --bin uniffi-bindgen generate --library target/release/libarithmetical.so --language kotlin --out-dir out
```

Then check out the `out` directory.

When using library mode, if multiple crates get built into the library that use UniFFI, all will have bindings generated for them.

Library mode comes with some extra requirements:
  - It must be run from within the cargo workspace of your project
  - Each crate must use exactly 1 UDL file when compiling the Rust library.  However, crates can have
    multiple UDL files as long as they ensure only one is used for any particular build,
    e.g. by using feature flags.
  - Rust sources must use `uniffi::include_scaffolding!` to include the scaffolding code.

## Running uniffi-bindgen with a single UDL file

Use the `generate` command to generate bindings by specifying a UDL file.

### Kotlin

From the `example/arithmetic` directory, run:
```
cargo run --bin uniffi-bindgen generate src/arithmetic.udl --language kotlin
```
then have a look at `src/uniffi/arithmetic/arithmetic.kt`

### Swift

Run
```
cargo run --bin uniffi-bindgen generate src/arithmetic.udl --language swift
```
then check out `src/arithmetic.swift`

Note that these commands could be integrated as part of your gradle/Xcode build process.

This is it, you have an MVP integration of UniFFI in your project.
