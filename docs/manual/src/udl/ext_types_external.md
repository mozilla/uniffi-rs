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

And further, assume that you have another crate called `consuming-crate` which would like to use
this dictionary. Inside `consuming-crate`'s UDL file you can reference `DemoDict` by using a
`typedef` with an `External` attribute, as shown below.

```idl
[External="demo-crate"]
typedef extern DemoDict;

// Now define our own dictionary which references the imported type.
dictionary ConsumingDict {
  DemoDict demo_dict;
  boolean another_bool;
};

```

(Note the special type `extern` used on the `typedef`. It is not currently enforced that the
literal `extern` is used, but we hope to enforce this later, so please use it!)

Inside `consuming-crate`'s Rust code you must `use` that struct as normal - for example,
`consuming-crate`'s `lib.rs` might look like:

```rust
use demo_crate::DemoDict;

pub struct ConsumingDict {
    demo_dict: DemoDict,
    another_bool: bool,
}

include!(concat!(env!("OUT_DIR"), "/consuming_crate.uniffi.rs"));
```

Your `Cargo.toml` must reference the external crate as normal.

The `External` attribute can be specified on dictionaries, enums and errors.

## Foreign bindings

The foreign bindings will also need to know how to access the external type,
which varies slightly for each language:

### Kotlin

For Kotlin, the generated code needs to import the external types from the
Kotlin module that corresponds to the Rust crate.  By default, UniFFI assumes
that the Kotlin module name matches the Rust crate name, but this can be
configured in `uniffi.toml` with an entry like this:

```
[bindings.kotlin.external_packages]
# Map the crate names from [External={name}] into Kotlin package names
rust-crate-name = "kotlin.package.name"
```

### Swift

For Swift, there are 2 ways to use the UniFFI-generated Swift code:
  - Compile all source files together in the same module.
  - Compile each source file into a different module.

By default, UniFFI assumes that the source files will be compiled together, so
external types will be automatically available and nothing needs to be configured.

If you compile each source file into a different module, then you need to
add configuration to `uniffi.toml`.

```
[bindings.swift.external_types]
in_different_modules = true
```

By default, UniFFI assumes the module name is the same as Rust crate name
specified in the UDL file (with hyphens replaced with underscores). You can
override this with the `bindings.swift.external_types.module_names` config key:

```
[bindings.swift.external_types.module_names]
my-rust-crate-name = "my_swift_module_name"
```
