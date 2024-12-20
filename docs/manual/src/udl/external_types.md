# Using types defined outside a UDL.

Often you need to refer to types described outside of this UDL - they
may be defined in a proc-macro in this crate or defined in an external crate.

You declare such types using:
```idl
typedef [type] [TypeName];
```
`TypeName` is then able to be used as a normal type in this UDL (ie, be returned from functions, in records, etc)

`type` indicates the actual type of `TypeName` and can be any of the following values:
* "enum" for Enums.
* "record", "dictionary" or "struct" for Records.
* "object", "impl" or "interface" for objects.
* "trait", "callback" or "trait_with_foreign" for traits.
* "custom" for Custom Types.

for example, if this crate has:
```rust
#[derive(::uniffi::Object)]
struct MyObject { ... }
```
our UDL could use this type with:
```
typedef interface MyObject;
```

# External Crates

The `[External="crate_name"]` attribute can be used whenever the type is in another crate - whether in UDL or in a proc-macro.

```
[External = "other_crate"]
typedef interface OtherObject;
```
