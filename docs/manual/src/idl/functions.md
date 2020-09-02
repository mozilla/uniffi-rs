# Functions

All top-level *functions* get exposed through the IDL's `namespace` block.
For example, if the crate's `lib.rs` file contains:

```rust
fn hello_world() -> String {
    "Hello World!".to_owned()
}
```

The IDL file will look like:

```idl
namespace Example {
    string hello_world();
}
