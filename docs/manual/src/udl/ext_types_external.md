# Declaring External Types

It is possible to use types defined by UniFFI in an external crate. For example, let's assume
that you have an existing crate named `demo_crate` with the following UDL:

```idl
dictionary DemoDict {
  string string_val;
  boolean bool_val;
};
```

Inside another crate, `consuming_crate`, you'd like to use this dictionary.
Inside `consuming_crate`'s UDL file you can reference `DemoDict` by using a
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

The `External` attribute can be specified on dictionaries, enums, errors.

## External interface and trait types

If the external type is an [Interface](./interfaces.md), then use the `[ExternalInterface]` attribute instead of `[External]`:

```idl
[ExternalInterface="demo_crate"]
typedef extern DemoInterface;
```

similarly for traits: use `[ExternalTrait]`.

## External procmacro types

The above examples assume the external types were defined via UDL.
If they were defined by procmacros, you need different attribute names:

- if `DemoDict` is implemented by a procmacro in `demo_crate`, you'd use `[ExternalExport=...]`
- for `DemoInterface` you'd use `[ExternalInterfaceExport=...]`

For types defined by procmacros in _this_ crate, see the [Attribute `[Rust=...]`](../ext_types.md)

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
