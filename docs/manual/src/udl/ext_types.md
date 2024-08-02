# External types

External types are types implemented by UniFFI but outside of this UDL file.

They are similar to, but different from [custom types](./custom_types.md) which wrap UniFFI primitive types.

But like custom types, external types are all declared using a `typedef` with attributes
giving more detail.

## Types in another crate

[There's a whole page about that!](./ext_types_external.md)

## Types from procmacros in this crate.

If your crate has types defined via `#[uniffi::export]` etc you can make them available
to the UDL file in your own crate via a `typedef` describing the concrete type.

```rust
#[derive(uniffi::Record)]
pub struct One {
    inner: i32,
}
```
you can use it in your UDL:

```idl
typedef record One;

namespace app {
    // use the procmacro type.
    One get_one(One? one);
}

```

Supported values:
* "enum", "trait", "callback", "trait_with_foreign"
* For records, either "record", "dictionary" or "struct"
* For objects, either "object", "impl" or "interface"

eg:
```
typedef enum MyEnum;
typedef interface MyObject;
```
etc.

Note that in 0.28 and prior, we also supported this capability with a `[Rust=]` attribute.
This attribute is deprecated and may be removed in a later version.
