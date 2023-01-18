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

## Running uniffi-bindgen

### Kotlin

Run
```
cargo run --bin uniffi-bindgen generate src/math.udl --language kotlin
```
then have a look at `src/uniffi/math/math.kt`

### Swift

Run
```
cargo run --bin uniffi-bindgen generate src/math.udl --language swift
```
then check out `src/math.swift`

Note that these commands could be integrated as part of your gradle/Xcode build process.

This is it, you have an MVP integration of UniFFI in your project.
