This is an experimental crate for generating JNI-based Kotlin bindings.

# Usage

## Rust crates

Crates that export a UniFFI interface should stay mostly the same,
but see below for extra requirements.
They should still depend on `uniffi`, except you can set `default-features = false`.
This disables the `macro-scaffolding` feature which speeds up compilation and reduces binary size.

## Rust top-level crate

A single, top-level crate should generate the JNI scaffolding.

Add dependencies to your `Cargo.toml`

```toml
[dependencies]
uniffi-bindgen-kotlin-jni-runtime = [version]

[build-dependencies]
uniffi-bindgen-kotlin-jni = [version]
```

Create a `build.rs` file that generates the scaffolding:

```rs
fn main() {
    uniffi_bindgen_kotlin_jni::generate_scaffolding();
}
```

Have your `lib.rs` file include the generated scaffolding:

```rs
include!(concat!(
    env!("OUT_DIR"),
    "/uniffi_bindgen_kotlin_jni.uniffi.rs"
));
```

## Kotlin bindings

Create a [bindgen binary](https://mozilla.github.io/uniffi-rs/latest/tutorial/foreign_language_bindings.html),
except have it depend on `uniffi-bindgen-kotlin-jni`.
The main function should call `uniffi_bindgen_kotlin_jni::main`.

Run your executable and use the `bindings` subcommand to generate bindings.
For example:

`cargo run -p [bindgen-binary-crate-name] bindings src:[library-crate-name] [out-dir]`

## Extra requriments compared to the normal macro code

* All exported items must be `pub`. This includes types, fields, functions, etc.
  Furthermore, types be publicly reachable (there must be a path that other crates can import).
* The top-level Rust crate must directly depend on all UniFFI crates
* The top-level Rust crate must depend on `uniffi-bindgen-kotlin-jni-runtime`
