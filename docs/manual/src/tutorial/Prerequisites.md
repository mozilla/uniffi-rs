# Prerequisites

This tutorial builds on our [`arithmetic`](https://github.com/mozilla/uniffi-rs/tree/main/examples/arithmetic) and (creatively-named) [`arithmetic-procmacro`](https://github.com/mozilla/uniffi-rs/tree/main/examples/arithmetic-procmacro) examples, which will be useful when we've omitted things.

Here we will be creating a `math` library - so we assume a `cargo new --lib math` environment.

## Add `uniffi` as a dependency and build-dependency

In your crate's `Cargo.toml` add:

```toml
[dependencies]
uniffi = { version = "[latest-version]" }

[build-dependencies]
uniffi = { version = "[latest-version]", features = [ "build" ] }
```

UniFFI has not reached version `1.0` yet.  Versions are typically specified as `0.[minor-version]`.

## Build your crate as a cdylib

Ensure your crate builds as a `cdylib` so looks something like
```toml
[lib]
crate-type = ["cdylib"]
name = "math" # This is our crate name in this tutorial
```
to your crate's `Cargo.toml`.

**Note:** You also need to add `staticlib` crate type if you target iOS.
