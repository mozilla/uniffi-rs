# Remote and External types

Remote and external types can help solve some advanced use-cases when using UniFFI.
They are grouped this section, since they're often used together.

# Remote types

"Remote types" refer to types defined in other crates that do not use UniFFI.
This normally means types from crates that you depend on but don't control.
Remote types require extra handling to use them in UniFFI APIs, because of Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits).
See https://github.com/mozilla/uniffi-rs/tree/main/examples/remote-types for example code.

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

Traits go through `#[uniffi::export]` rather than `#[uniffi::remote]`, so they use a `remote`
argument instead:

```rust
// As above, this mirrors the definition from the remote crate; UniFFI generates the scaffolding
// but doesn't output the trait itself.
#[uniffi::export(remote)]
pub trait Logger: Send + Sync {
    fn log(&self, message: String);
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

[Trait, Remote]
interface Logger {
    void log(string message);
};
```

## Remote traits can't be foreign

Supporting foreign implementations of a trait requires UniFFI to change the trait,
which is impossible for a trait defined externally. `#[uniffi::export(remote, foreign)]` and
`[Trait, Remote, WithForeign]` are therefore errors — only Rust can implement a remote trait.

At time of writing, the change made to a non-remote trait is a new method:
```
#[doc(hidden)]
fn uniffi_foreign_handle(&self) { ... }
```

# External Types

"External types" refer to types defined in other crates that use UniFFI.
This normally means types from other crates in your workspace.

Proc-macros typically use external types automatically, but UDL needs them described.

## UDL

Suppose you depend on the `DemoDict` type from another UniFFIed crate in your workspace.
You can reference this type by using the `[External]` attribute to wrap a typedef describing the concrete type.

```idl
[External="demo_crate"]
typedef dictionary DemoDict;

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
* Custom types: `custom`

# Special cases for remote types

There are a few cases where remote types require extra handling in addition to the rules above.

## Remote + External types

Types that are remote and external require a `use_remote_type!` macro call.

If `crate_a` defines [IpAddr](https://doc.rust-lang.org/std/net/enum.IpAddr.html) as a remote type, then `crate_b` can use that type with the following Rust code:

```rust
uniffi::use_remote_type!(crate_a::IpAddr);
```

## UDL

UDL-users will also need to add the external type definition:

```idl
[External="crate_a"]
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
