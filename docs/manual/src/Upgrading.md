# Upgrading v0.28.x -> v0.29.x

We've made a number of breaking changes in this release, particularly
to:

* Custom types: UniffiCustomTypeConverter has been removed.
* External types: `extern` has been removed; you must describe the type.

## Custom types

Custom types still implemented via the `UniffiCustomTypeConverter` trait must move to proc-macros.

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

```rust
uniffi::custom_type!(NewCustomType, BridgeType, {
    try_lift: |val| { Ok(...) },
    lower: |obj| { ... },
})
```

The `custom_type!` macro is more flexible than the old system - eg, the closures can be omitted in many cases where `From` and `Into` exist.
See the [Custom Types](./types/custom_types.md) for details.

## External Types

External types can no longer be described in UDL via `extern` - instead, you must specify the type.

For example:
```
[External="crate_name"]
typedef extern MyEnum;
```
is no longer accepted - you must use, eg:
```
[External="crate_name"]
typedef enum MyEnum;
```

Edge-cases broken include:

* Different variations of the `External` attribute (eg, `[ExternalInterface]`) are no longer supported; eg, `[ExternalInterface=".."] typedef extern ...` becomes `[External=".."] typedef interface ...` )
* The `[Rust=..]` attribute has been removed - you should just remove the attribute entirely.

See [Remote and External Types](./types/remote_ext_types.md) for more detail.

## External Custom Types

Previously you could describe an external Custom Type `Url` in UDL as:
```
[External="crate_name"]
typedef extern Url;
```

But now you must use:
```
[Custom="crate_name"]
typedef string Url; // replace `string` with any appropriate type.
```

## Remote Types

Remote types in UDL now need the `[Remote]` attribute:
```
[Remote]
dictionary RemoteRecord {
    ...
};
```

The macros `ffi_converter_forward` and all `use_*` macros (eg, `use_udl_record!`, `use_udl_object!`, `use_udl_enum!` etc)
are now unnecessary so have been removed.

See [Remote and External Types](./types/remote_ext_types.md) for more detail.

## Shared Rust/UDL types

The `Rust` attribute has been removed - use the same typedef syntax described above for External Types.

```
[Rust="record"]
typedef extern One;
```
becomes a `typedef` with no attributes
```
typedef record One;
```
