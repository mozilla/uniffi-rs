# Experimental: Attributes and Derives

UniFFI is in the process of having its interface definition mechanism rewritten to avoid the
duplication of function signatures and type definitions between Rust code and the UDL file (and the
possibility for the two to go out of sync). The new interface definition mechanism is based on
[Procedural Macros][] (proc-macros), specifically the attribute and derive macros.

This rewrite is not yet complete, but can already have UniFFI extract some kinds of definitions out
of your Rust code, in addition to what is declared in the UDL file. However, you have to make sure
that the UDL file is still valid on its own: All types referenced in fields, parameter and return
types in UDL must also be declared in UDL.

[Procedural Macros]: https://doc.rust-lang.org/reference/procedural-macros.html

**⚠ Warning ⚠** As the page title says, this is experimental. Bugs are expected and if you want to
try it is recommended that you use `uniffi` as a git dependency so you don't run into issues that
are already fixed.

## Build workflow

Before any of the things discussed below work, make sure to update your bindings generation steps
to start with a build of your library and add
`--lib-file <CARGO WORKSPACE>/target/<TARGET>/<BUILT CDYLIB OR STATICLIB>` to your `uniffi-bindgen`
command line invocation.

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

User-defined types also have to be (re-)exported from a module called `uniffi_types` at the crate
root. This is required to ensure that a given type name always means the same thing across all uses
of `#[uniffi::export]` across the whole module tree.

```rust
mod uniffi_types {
    pub(crate) use path::to::MyObject;
}
```

## The `uniffi::Record` derive

The `Record` derive macro exposes a `struct` with named fields over FFI. All types that are
supported as parameter and return types by `#[uniffi::export]` are also supported as field types
here.

It is permitted to use this macro on a type that is also defined in the UDL file, as long as all
field types are UniFFI builtin types; user-defined types might be allowed in the future. You also
have to maintain a consistent field order between the Rust and UDL files (otherwise compilation
will fail).

```rust
#[derive(uniffi::Record)]
pub struct MyRecord {
    pub field_a: String,
    pub field_b: Option<Arc<MyObject>>,
}
```

## The `uniffi::Enum` derive

The `Enum` derive macro works much like the `Record` derive macro. Any fields inside variants must
be named. All types that are supported as parameter and return types by `#[uniffi::export]` are
also supported as field types.

It is permitted to use this macro on a type that is also defined in the UDL file as long as the
two definitions are equal in the names and ordering of variants and variant fields, and any field
types inside variants are UniFFI builtin types; user-defined types might be allowed in the future.

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Fieldless,
    WithFields {
        foo: u8,
        bar: Vec<i32>,
    },
}
```

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

## The `uniffi::Error` derive

The `Error` derive registers a type as an error and can be used on any enum that the `Enum` derive also accepts.
By default, it exposes any variant fields to the foreign code.
This type can then be used as the `E` in a `Result<T, E>` return type of an exported function or method.
The generated foreign function for an exported function with a `Result<T, E>` return type
will have the result's `T` as its return type and throw the error in case the Rust call returns `Err(e)`.

```rust
#[derive(uniffi::Error)]
pub enum MyError {
    MissingInput,
    IndexOutOfBounds {
        index: u32,
        size: u32,
    }
    Generic {
        message: String,
    }
}

#[uniffi::export]
fn do_thing() -> Result<(), MyError> {
    // ...
}
```

You can also use the helper attribute `#[uniffi(flat_error)]` to expose just the variants but none of the fields.
In this case the error will be serialized using Rust's `ToString` trait
and will be accessible as the only field on each of the variants.
For flat errors your variants can have unnamed fields,
and the types of the fields don't need to implement any special traits.

```rust
#[derive(uniffi::Error)]
#[uniffi(flat_error)]
pub enum MyApiError {
    Http(reqwest::Error),
    Json(serde_json::Error),
}

// ToString is not usually implemented directly, but you get it for free by implementing Display.
// This impl could also be generated by a proc-macro, for example thiserror::Error.
impl std::fmt::Display for MyApiError {
    // ...
}

#[uniffi::export]
fn do_http_request() -> Result<(), MyApiError> {
    // ...
}
```

## Other limitations

In addition to the per-item limitations of the macros presented above, there is also currently a
global restriction: You can only use the proc-macros inside a crate whose name is the same as the
namespace in your UDL file. This restriction will be lifted in the future.
