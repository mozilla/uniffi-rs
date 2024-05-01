# Remote and External types

Remote and external types can help solve some advanced use-cases when using UniFFI.
They are grouped this section, since they're often used together.

# Remote types

"Remote types" refer to types defined in other crates that do not use UniFFI.
This normally means types from crates that you depend on but don't control.
Remote types require extra handling to use them in UniFFI APIs, because of Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits).
See https://github.com/mozilla/uniffi-rs/tree/main/examples/log-formatter for example code.

In general, using remote types in UniFFI requires writing a type definition that mirrors the real definition found in the remote crate.

## Proc-macros

```rust

// Type aliases can be used to give remote types nicer names when exposed in the UniFFI api.
type LogLevel = log::Level;

// Write a definition that mirrors the definition from the remote crate and wrap it with `[uniffi::remote(<kind>)]`.
//
// - UniFFI will generate the FFI scaffolding code for the item, but will not output the item itself
//   (since the real item is defined in the remote crate).
// - `<kind>` can be any parameter that's valid for `uniffi::derive()`.
#[uniffi::remote(Enum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
```

## UDL

Wrap the definition with `[Remote]` attribute:

```idl
[Remote]
enum LogLevel {
    "Error",
    "Warn",
    "Info",
    "Debug",
    "Trace",
};
```

# External Types

"External types" refer to types defined in other crates that use UniFFI.
This normally means types from other crates in your workspace.

## Proc-macros

Proc-macro-based code can use external types automatically, without any extra code.

## UDL

Suppose you depend on the `DemoDict` type from another UniFFIed crate in your workspace.
You can reference this type by using the `[External]` attribute to wrap a typedef describing the concrete type.

```idl
[External]
typedef dictionary One;

// Now define our own dictionary which references the external type.
dictionary ConsumingDict {
  DemoDict demo_dict;
  boolean another_bool;
};
```

Supported values for the typedef type:

* Enums: `enum`
* Records: `record`, `dictionary` or `struct`
* Objects: `object`, `impl` or `interface`
* Traits: `trait`, `callback` or `trait_with_foreign`

# Special cases for remote types

There are a few cases where remote types require extra handling in addition to the rules above.

## Remote + External types

Types that are remote and external require a `use_remote_type!` macro call.

If `crate_a` defines [IpAddr](https://doc.rust-lang.org/std/net/enum.IpAddr.html) as a remote type, then `crate_b` can use that type with the following Rust code:

```rust
uniffi::use_remote_type!(IpAddr, crate_a);
```

## UDL

UDL-users will also need to add the external type definition:

```idl
[External]
typedef enum IpAddr;
```

## Remote custom types

Types that are remote and custom require using the `remote` attribute with the `custom_type!` macro.

```rust

uniffi::custom_type!(StructFromOtherCrate, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| StructFromOtherCrate::try_from_string(s),
});
```

## Foreign bindings

The foreign bindings will also need to know how to access the external type,
which varies slightly for each language:

### Kotlin

For Kotlin, "library mode" generation with `generate --library [path-to-cdylib]` is recommended when using external types.
If you use `generate [udl-path]` then the generated code needs to know how to import
the external types from the Kotlin module that corresponds to the Rust crate.
By default, UniFFI assumes that the Kotlin module name matches the Rust crate name, but this can be configured in `uniffi.toml` with an entry like this:

```
[bindings.kotlin.external_packages]
# Map the crate names from [External={name}] into Kotlin package names
rust-crate-name = "kotlin.package.name"
```

### Swift

For Swift, you must compile all generated `.swift` files together in a single
module since the generate code expects that it can access external types
without importing them.
