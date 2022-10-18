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
name = "run-uniffi-bindgen"
path = "run-uniffi-bindgen.rs"
```

Create `run-uniffi-bindgen.rs`:
```rust
fn main() {
    uniffi::run_uniffi_bindgen().unwrap()
}
```

You can now run `uniffi-bindgen` from your project using `cargo run --bin run-uniffi-bindgen`

### Multi-crate workspaces

If your project consists of multiple crates in a Cargo workspace, then the process outlined above would require you
creating a binary for each crate that uses UniFFI.  You can avoid this by creating a single crate named
`run-uniffi-bindgen` that depends on `uniffi` and has the `run-uniffi-bindgen.rs` binary.  Then your can run
`uniffi-bindgen` from any create in your project using `cargo run -p run-uniffi-bindgen`


## Running uniffi-bindgen

### Kotlin

Run
```
cargo run --bin run-uniffi-bindgen generate src/math.udl --language kotlin
```
then have a look at `src/uniffi/math/math.kt`

### Swift

Run
```
cargo run --bin run-uniffi-bindgen generate src/math.udl --language swift
```
then check out `src/math.swift`

Note that these commands could be integrated as part of your gradle/Xcode build process.

This is it, you have an MVP integration of UniFFI in your project.
