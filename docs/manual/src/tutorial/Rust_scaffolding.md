# Rust scaffolding

## Rust scaffolding code

Now we generate some Rust helper code to make the `add` method available to foreign-language bindings.  

First, add `uniffi` to your crate dependencies: it is the runtime support code that powers uniffi's serialization of data types across languages:

<!-- TODO: Use a published version -->
```toml
[dependencies]
uniffi = { git = "https://github.com/mozilla/uniffi-rs", branch = "main" }
```

Then let's add `uniffi_build` to your build dependencies: it generates the Rust scaffolding code that exposes our Rust functions as a C-compatible FFI layer.

<!-- TODO: Use a published version -->
```toml
[build-dependencies]
uniffi_build = { git = "https://github.com/mozilla/uniffi-rs", branch = "main" }
```

Then create a `build.rs` file next to `Cargo.toml` that will use `uniffi_build`:

```rust
fn main() {
    uniffi_build::generate_scaffolding("./src/math.idl").unwrap();
}
```

**Note:** This is the equivalent of calling (and does it under the hood) `uniffi-bindgen scaffolding src/math.idl --out-dir <OUT_DIR>`.

Lastly, we include the generated scaffolding code in our `lib.rs`:
```rust
include!(concat!(env!("OUT_DIR"), "/math.uniffi.rs"));
```
**Note:** The file name is always `<namespace>.uniffi.rs`.

Great! `add` is ready to see the outside world!
