# The Interface Module

We describe the interface to be exposed by your crate using a restricted subset of Rust syntax.
In this case, we are only playing with primitive types (`u32`) and not custom data structures but we still want to expose the `add` method. We start by wrapping it in an inline submodule, like this:

```rust
mod math {
  pub fn add(a: u32, b: u32) -> u32 {
    a + b
  }
};
```

Here you can note multiple things:
- The interface is wrapped in an inline module named `math`. This will be the default name of your Kotlin/Swift package.
- The `add` function is inside the interface declaration module and is marked as `pub`, so it will
  be exposed as part of the foreign-language bindings.

