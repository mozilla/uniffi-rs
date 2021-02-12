# Rust scaffolding

To expose the `add` method for use by foreign-language bindings, your crate will also
need to contain various bits of helper code that we call the *Rust scaffolding*.
Luckily this can all be generated automatically by UniFFI!

First, add `uniffi` to your crate dependencies:

```toml
[dependencies]
uniffi = "0.7"
```

Important note: the `uniffi` version must be the same as the `uniffi-bindgen` command-line tool installed on your system.

Next, use the `declare_interface` macro on the interface module you defined
in the previous step:

```rust
#[uniffi::declare_interface]
mod math {
  pub fn add(a: u32, b: u32) -> u32 {
    a + b
  }
};
```

Finally, `cargo build` your crate to check that things are working correctly.
If you have used unsupported syntax in your interface definition module then UniFFI
will produce an error during the build.

Great! `add` is ready to see the outside world!
