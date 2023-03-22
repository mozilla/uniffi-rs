# Prerequisites

## Add `uniffi` as a dependency and build-depedency

In your crate's `Cargo.toml` add:

```toml
[dependencies]
uniffi = { version = "[latest-version]" }

[build-dependencies]
uniffi = { version = "[latest-version]", features = [ "build" ] }
```

UniFFI has not reached version `1.0` yet.  Versions are typically specified as "0.[minor-version]".

## Build your crate as a cdylib

Ensure your crate builds as a `cdylib` by adding
```toml
[lib]
crate-type = ["cdylib"]
name = "<library name>"
```
to your crate's `Cargo.toml`.

**Note:** You also need to add `staticlib` crate type if you target iOS.
