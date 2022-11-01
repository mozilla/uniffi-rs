# Experimental: Attributes and Derives

UniFFI is in the process of having its interface definition mechanism rewritten to avoid the
duplication of function signatures and type definitions between Rust code and the UDL file.
The new interface definition mechanism is based on [Procedural Macros][] (proc-macros),
specifically the attribute and derive macros.

This rewrite is not yet complete, but can already have UniFFI extract some kinds of definitions out
of your Rust code, in addition to what is declared in the UDL file. However, you have to make sure
that the UDL file is still valid on its own: All types referenced in fields, parameter and return
types in UDL must also be declared in UDL.

[Procedural Macros]: https://doc.rust-lang.org/reference/procedural-macros.html

## The `#[uniffi::export]` attribute

The most important proc-macro is the `export` attribute. It can be used on functions and `impl`
blocks to make UniFFI aware of them.

```rust
#[uniffi::export]
fn hello_ffi() {
    println!("Hello from Rust!");
}

// Corresponding UDL:
//
// interface MyObject {};
struct MyObject {
    // ...
}

#[uniffi::export]
impl MyObject {
    // All methods must have a `self` argument
    fn method_a(&self) {
        // ...
    }

    // Arc<Self> is also supported
    fn method_b(self: Arc<Self>) {
        // ...
    }
}
```

Most UniFFI [builtin types](../udl/builtin_types.md) can be used as parameter and return types.
When a type is not supported, you will get a clear compiler error about it.

User-defined types are also supported in a limited manner: records (structs with named fields,
`dictionary` in UDL) and enums can be used when the corresponding derive macro is used at
their definition. Opaque objects (`interface` in UDL) can always be used regardless of whether they
are defined in UDL and / or via derive macro; they just need to be put inside an `Arc` as always.

## The `uniffi::Record` derive

The `Record` derive macro exposes a `struct` with named fields over FFI. All types that are
supported as parameter and return types by `#[uniffi::export]` are also supported as field types
here.

It is permitted to use this macro on a type that is also defined in the UDL file, as long as all
field types are UniFFI builtin types; user-defined types might be allowed in the future. You also
have to maintain a consistent field order between the Rust and UDL files (otherwise compilation
will fail).

## The `uniffi::Enum` derive

The `Enum` derive macro works much like the `Record` derive macro. Any fields inside variants must
be named. All types that are supported as parameter and return types by `#[uniffi::export]` are
also supported as field types.

It is permitted to use this macro on a type that is also defined in the UDL file as long as the
two definitions are equal in the names and ordering of variants and variant fields, and any field
types inside variants are UniFFI builtin types; user-defined types might be allowed in the future.

## The `uniffi::Object` derive

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

## Other limitations

In addition to the per-item limitations of the macros presented above, there is also currently a
global restriction: You can only use the proc-macros inside of a crate whose name is the same as
the namespace in your UDL file. This restriction will be lifted in the future.
