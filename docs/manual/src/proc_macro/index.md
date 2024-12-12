# Procedural Macros: Attributes and Derives

UniFFI allows you to define your function signatures and type definitions directly in your Rust
code, avoiding the need to duplicate them in a UDL file and so avoiding the possibility for the two to get out of sync.
This  mechanism is based on [Procedural Macros][] (proc-macros), specifically the attribute and derive macros.

You can have this mechanism extract some kinds of definitions out of your Rust code,
in addition to what is declared in the UDL file. However, you have to make sure
that the UDL file is still valid on its own: All types referenced in fields, parameter and return
types in UDL must also be declared in UDL.

Further, using this capability probably means you still need to refer to the UDL documentation,
because at this time, that documentation tends to conflate the UniFFI type model and the
description of how foreign bindings use that type model. For example, the documentation for
a UDL interface describes both how it is defined in UDL and how Swift and Kotlin might use
that interface. The latter is relevant even if you define the interface using proc-macros
instead of in UDL.

[Procedural Macros]: https://doc.rust-lang.org/reference/procedural-macros.html

**⚠ Warning ⚠** This facility is relatively new, so things may change often. However, this remains
true for all of UniFFI, so proceed with caution and the knowledge that things may break in the future.

## Build workflow

Be sure to use library mode when using UniFFI proc-macros (See the [Foreign language bindings docs](../tutorial/foreign_language_bindings.md) for more info).

If your crate's API is declared using only proc-macros and not UDL files, call the `uniffi::setup_scaffolding` macro at the top of your source code:

```rust
uniffi::setup_scaffolding!();
```

**⚠ Warning ⚠** Do not call both `uniffi::setup_scaffolding!()` and `uniffi::include_scaffolding!!()` in the same crate.

## The `#[uniffi::export]` attribute

The most important proc-macro is the `export` attribute. It can be used on functions, `impl`
blocks, and `trait` definitions to make UniFFI aware of them.

```rust
#[uniffi::export]
fn hello_ffi() {
    println!("Hello from Rust!");
}
```

For more details:
* [Records](./records.md)
* [Enums](./enumerations.md)
* [Interfaces](./interfaces.md)
* [Functions, constructors, methods](./functions.md)
* [Errors](./errors.md)

## The `uniffi::Object` derive to extend interfaces defined in UDL

This derive can be used to replace an `interface` definition in UDL. Every object type must have
*either* an `interface` definition in UDL *or* use this derive macro. However, `#[uniffi::export]`
can be used on an impl block for an object type regardless of whether this derive is used. You can
also mix and match, and define some method of an object via proc-macro while falling back to UDL
for methods that are not supported by `#[uniffi::export]` yet; just make sure to use separate
`impl` blocks:

```idl
// UDL file

interface Foo {
    void method_a();
};
```

```rust
// Rust file

// Not deriving uniffi::Object since it is defined in UDL
struct Foo {
    // ...
}

// Implementation of the method defined in UDL
impl Foo {
    fn method_a(&self) {
        // ...
    }
}

// Another impl block with an additional method
#[uniffi::export]
impl Foo {
    fn method_b(&self) {
        // ...
    }
}
```

## The `uniffi::custom_type` and `uniffi::custom_newtype` macros

See the general [documentation for Custom Types](../types/custom_types.md), which apply equally to proc-macros as to UDL.


## The `#[uniffi::export(callback_interface)]` attribute

`#[uniffi::export(callback_interface)]` can be used to export a [callback interface](../types/callback_interfaces.md) definition.
This allows the foreign bindings to implement the interface and pass an instance to the Rust code.

```rust
#[uniffi::export(callback_interface)]
pub trait Person {
    fn name() -> String;
    fn age() -> u32;
}

// Corresponding UDL:
// callback interface Person {
//     string name();
//     u32 age();
// }
```

## Conditional compilation

`uniffi::constructor|method]` will work if wrapped with `cfg_attr` attribute:
```rust
#[cfg_attr(feature = "foo", uniffi::constructor)]
```
Other attributes are not currently supported, see [#2000](https://github.com/mozilla/uniffi-rs/issues/2000) for more details.

## Mixing UDL and proc-macros

If you use both UDL and proc-macro generation, then your crate name must match the namespace in your
UDL file. This restriction will be lifted in the future.
