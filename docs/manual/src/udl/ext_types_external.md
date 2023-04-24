# Declaring External Types

*Note: The facility described in this document is not yet available for all foreign language
bindings.*

It is possible to use types defined by UniFFI in an external crate. For example, let's assume
that you have an existing crate named `demo_crate` with the following UDL:

```idl
dictionary DemoDict {
  string string_val;
  boolean bool_val;
};
```

And further, assume that you have another crate called `consuming_crate` which would like to use
this dictionary. Inside `consuming_crate`'s UDL file you can reference `DemoDict` by using a
`typedef` with an `External` attribute, as shown below.

```idl
[External="demo_crate"]
typedef extern DemoDict;

// Now define our own dictionary which references the imported type.
dictionary ConsumingDict {
  DemoDict demo_dict;
  boolean another_bool;
};

```

(Note the special type `extern` used on the `typedef`. It is not currently enforced that the
literal `extern` is used, but we hope to enforce this later, so please use it!)

Inside `consuming_crate`'s Rust code you must `use` that struct as normal - for example,
`consuming_crate`'s `lib.rs` might look like:

```rust
use demo_crate::DemoDict;

pub struct ConsumingDict {
    demo_dict: DemoDict,
    another_bool: bool,
}

uniffi::include_scaffolding!("consuming_crate");
```

Your `Cargo.toml` must reference the external crate as normal.

The `External` attribute can be specified on dictionaries, enums and errors.

## External interface types

If the external type is an [Interface](./interfaces.md), then use the `[ExternalInterface]` attribute instead of `[External]`:

```idl
[ExternalInterface="demo_crate"]
typedef extern DemoInterface;
```

## Foreign bindings

The foreign bindings will also need to know how to access the external type,
which varies slightly for each language:

### Kotlin

For Kotlin, "crate mode" generation with `generate --crate [crate-name]` is recommended when using external types.
If you use `generate [udl-path]` then the generated code needs to know how to import
the external types from the Kotlin module that corresponds to the Rust crate.
By default, UniFFI assumes that the Kotlin module name matches the Rust crate name, but this can be configured in `uniffi.toml` with an entry like this:

```
[bindings.kotlin.external_packages]
# Map the crate names from [External={name}] into Kotlin package names
rust-crate-name = "kotlin.package.name"
```

See the [`ext-types` fixture](https://github.com/mozilla/uniffi-rs/blob/main/fixtures/ext-types/lib/uniffi.toml)
for an example

### Swift

For Swift, you must compile all generated `.swift` files together in a single
module since the generate code expects that it can access external types
without importing them.
