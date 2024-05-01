# Remote, Custom and External types

Remote, custom, and external types can help solve some advanced use-cases when using UniFFI.
They are grouped this section, since they're often used together.

# Remote types

Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits) places restrictions on implementing traits for types defined in other crates.
Because of that, Remote types require extra handling to use them in UniFFI APIs.

- See https://github.com/mozilla/uniffi-rs/tree/main/examples/log-formatter for example code.

## Proc-macros

```rust

// Type aliases can be used to give remote types nicer names when exposed in the UniFFI api.
type LogLevel = log::Level;

// Wrap the definition of the type with `[uniffi::remote(<kind>)]`.
//
// - The definiton should match the definition on the remote side.
// - UniFFI will generate the FFI scaffolding code for the item, but will not output the item itself
//   (since the real item is defined in the remote crate).
// - `<kind>` can be any parameter that's valid for `uniffi::derive()`.
#[uniffi::remote(Enum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    "Trace",
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

It is possible to use types defined by UniFFI in an external crate. For example, let's assume
that you have an existing crate with the following UDL:

```idl
dictionary DemoDict {
  string string_val;
  boolean bool_val;
};

namespace demo_crate {
   ...
};
```

Inside another crate, `consuming_crate`, you'd like to use this dictionary.
Inside `consuming_crate`'s UDL file you can reference `DemoDict` by using the
`[External=<namespace>]` attribute to wrap an empty definition.

```idl
[External="demo_crate"]
dictionary DemoDict { }

// Now define our own dictionary which references the imported type.
dictionary ConsumingDict {
  DemoDict demo_dict;
  boolean another_bool;
};
```

## External + Remote types

If a type is both External and Remote, then it requires some special handling to work around Rust's
orphan rules.  Call the `use_remote_type!` macro to handle this.  `use_remote_type!` works with both
UDL and proc-macro based generation.

```rust
uniffi::use_remote_type!(RemoteType, crate_with_the_remote_type_implementation);
```

## Proc-macros

Proc-macros do not need to take any special action to use external types (other than the external +
remote) case above.

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
