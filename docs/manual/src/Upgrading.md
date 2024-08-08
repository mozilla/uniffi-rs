# v0.28.x -> v0.29.x

## Custom types

Custom types are now implemented using a macro, rather than implementing the
`UniffiCustomTypeConverter` trait.

Before:

```rust
impl UniffiCustomTypeConverter for NewCustomType {
    type Builtin = AnyExstingUniffiType;

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
uniffi::custom_type!(NewCustomType, AnyExstingUniffiType, {
    try_from_existing: |val| { Ok(...) },
    into_existing: |obj| { ... },
})
```

The custom_type macro is more flexible than the old system.  For example, the `try_from_existing`
and `into_existing` can be omitted in many cases.  If `try_from_existing` is omitted, then
`TryFrom<AnyExstingUniffiType>` will be used instead.  Likewise, if `into_existing` is omitted, then
`From<NewCustomType>` will be used instead. See the [Custom Types](./udl/custom_types.md) for details and
other features of the new macro.
