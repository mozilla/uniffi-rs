# External types

External types are types implemented by UniFFI but outside of this UDL file.

They are similar to, but different from [custom types](./custom_types.md) which wrap UniFFI primitive types.

But like custom types, external types are all declared using a `typedef` with attributes
giving more detail.

## Types in another crate

[There's a whole page about that!](./ext_types_external.md)

## Types from procmacros in this crate.

If your crate has types defined via `#[uniffi::export]` etc you can make them available
to the UDL file in your own crate via a `typedef` with a `[Rust=]` attribute. Eg, your Rust
might have:

```rust
#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}
```
you can use it in your UDL:

```idl
[Rust="record"]
typedef extern One;

namespace app {
    // use the procmacro type.
    One get_one(One? one);
}

```

Supported values:
*  "enum", "trait", "callback"
* For records, either "record" or "dictionary"
* For objects, either "object" or "interface"
