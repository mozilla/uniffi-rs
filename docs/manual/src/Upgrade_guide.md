# v0.28.x -> v0.29.x

## Custom types

Custom types are now implemented using a macro, rather than implementing the
`UniffiCustomTypeConverter` trait.

Before:

```rust
impl UniffiCustomTypeConverter for MyType {
    type Builtin = UniFFIBuiltinType;

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
uniffi::custom_type!(MyType, UniFFIBuiltinType, {
    try_into_custom: |val| { ... },
    from_custom: |obj| { ... },
})
```

The custom_type macro is more flexible than the old system.  For example, the `try_into_custom` and
`from_custom` can be omitted in many cases, and will use the `TryInto` and `From` traits by default.
See the [Custom Types](./udl/custom_types.md) for details and other features of the new macro.
