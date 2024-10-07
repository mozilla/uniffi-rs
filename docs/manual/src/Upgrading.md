# v0.28.x -> v0.29.x

## Custom types

Custom types are now implemented using a macro, rather than implementing the `UniffiCustomTypeConverter` trait.
We did this to help fix some edge-cases with custom types wrapping types from other crates (eg, Url).

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

The term "bridge type" replaces "builtin type".
The reason fo this is that this type does not actually need to be a builtin type.
Records, Enums, and Objects are also supported.

After:

```
uniffi::custom_type!(NewCustomType, BridgeType, {
    try_from_bridge_type: |val| { Ok(...) },
    into_bridge_type: |obj| { ... },
})
```

The custom_type macro is more flexible than the old system. 
For example, the `try_from_bridge_type` and `into_bridge_type` can be omitted in many cases
If `into_bridge_type` is omitted, then `Into<BridgeType>` will be used instead.
If `try_from_bridge_type` is omitted, then `TryInto<NewCustomType>` will be used instead.
The non-symmetry is slightly awkward, but `TryInto` is better to use than `TryFrom` because of the blanket impls in the standard library and we decided that writing `try_from_bridge_type` would be more natural for users invoking the macro.

See the [Custom Types](./udl/custom_types.md) for details and other features of the new macro.
