# v0.28.x -> v0.29.x

## Custom types

Custom types are now implemented using a macro rather than implementing the `UniffiCustomTypeConverter` trait,
addressing some edge-cases with custom types wrapping types from other crates (eg, Url).

Before:

```rust
impl UniffiCustomTypeConverter for NewCustomType {
    type Builtin = BridgeType;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        ...
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        ...
    }
}
```

After:

```
uniffi::custom_type!(NewCustomType, BridgeType, {
    try_lift: |val| { Ok(...) },
    lower: |obj| { ... },
})
```

The `custom_type!` macro is more flexible than the old system - eg, the closures can be omitted in many cases where `From` and `Into` exist.
See the [Custom Types](./udl/custom_types.md) for details.
