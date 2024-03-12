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

// Corresponding UDL:
//
// interface MyObject {};
#[derive(uniffi::Object)] 
struct MyObject {
    // ...
}

#[uniffi::export]
impl MyObject {
    // Constructors need to be annotated as such.
    // The return value can be either `Self` or `Arc<Self>`
    // It is treated as the primary constructor, so in most languages this is invoked with
    `MyObject()`.
    #[uniffi::constructor]
    fn new(argument: String) -> Arc<Self> {
        // ...
    }

    // Constructors with different names are also supported, usually invoked
    // as `MyObject.named()` (depending on the target language)
    #[uniffi::constructor]
    fn named() -> Arc<Self> {
        // ...
    }

    // All functions that are not constructors must have a `self` argument
    fn method_a(&self) {
        // ...
    }

    // Returning objects is also supported, either as `Self` or `Arc<Self>`
    fn method_b(self: Arc<Self>) {
        // ...
    }
}

// Corresponding UDL:
// [Trait]
// interface MyTrait {};
#[uniffi::export]
trait MyTrait {
    // ...
}

// Corresponding UDL:
// [Trait, WithForeign]
// interface MyTrait {};
#[uniffi::export(with_foreign)]
trait MyTrait {
    // ...
}
```


All owned [builtin types](../udl/builtin_types.md) and user-defined types can be used as arguments
and return types.

Arguments and receivers can also be references to these types, for example:

```rust
// Input data types as references
#[uniffi::export]
fn process_data(a: &MyRecord, b: &MyEnum, c: Option<&MyRecord>) {
    ...
}

#[uniffi::export]
impl Foo {
  // Methods can take a `&self`, which will be borrowed from `Arc<Self>`
  fn some_method(&self) {
    ...
  }
}

// Input foo as an Arc and bar as a reference
fn call_both(foo: Arc<Foo>, bar: &Foo) {
  foo.some_method();
  bar.some_method();
}
```

The one restriction is that the reference must be visible in the function signature.  This wouldn't
work:

```rust
type MyFooRef = &'static Foo;

// ERROR: UniFFI won't recognize that the `foo` argument is a reference.
#[uniffi::export]
fn do_something(foo: MyFooRef) {
}
```

### Renaming functions, methods and constructors

A single exported function can specify an alternate name to be used by the bindings by specifying a `name` attribute.

```rust
#[uniffi::export(name = "something")]
fn do_something() {
}
```
will be exposed to foreign bindings as a namespace function `something()`

You can also rename constructors and methods:
```rust
#[uniffi::export]
impl Something {
    // Set this as the default constructor by naming it `new`
    #[uniffi::constructor(name = "new")]
    fn make_new() -> Arc<Self> { ... }

    // Expose this as `obj.something()`
    #[uniffi::method(name = "something")]
    fn do_something(&self) { }
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
    // Fields can have a default value.
    // Currently, only string, integer, float and boolean literals are supported as defaults.
    #[uniffi(default = "hello")]
    pub greeting: String,
    #[uniffi(default = true)]
    pub some_flag: bool,
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
    WithValue = 3,
}
```

### Variant Discriminants

Variant discriminants are accepted by the macro but how they are used depends on the bindings.

For example this enum:

```rust
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Foo = 3,
    Bar = 4,
}
```

would give you in Kotlin & Swift:

```swift
// kotlin
enum class MyEnum {
    FOO,
    BAR;
    companion object
}
// swift
public enum MyEnum {
    case foo
    case bar
}
```

which means you cannot use the platforms helpful methods like `value` or `rawValue` to get the underlying discriminants. Adding a `repr` will allow the type to be defined in the foreign bindings.

For example:

```rust
// added the repr(u8), also u16 -> u64 supported
#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum MyEnum {
    Foo = 3,
    Bar = 4,
}
```

will now generate:

```swift
// kotlin
enum class MyEnum(val value: UByte) {
    FOO(3u),
    BAR(4u);
    companion object
}

// swift
public enum MyEnum : UInt8 {
    case foo = 3
    case bar = 4
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

## The `uniffi::custom_type` and `uniffi::custom_newtype` macros

There are 2 macros available which allow procmacros to support "custom types" as described in the
[UDL documentation for Custom Types](../udl/custom_types.md)

The `uniffi::custom_type!` macro requires you to specify the name of the custom type, and the name of the
builtin which implements this type. Use of this macro requires you to manually implement the
`UniffiCustomTypeConverter` trait for for your type, as shown below.
```rust
pub struct Uuid {
    val: String,
}

// Use `Uuid` as a custom type, with `String` as the Builtin
uniffi::custom_type!(Uuid, String);

impl UniffiCustomTypeConverter for Uuid {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Uuid { val })
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.val
    }
}
```

There's also a `uniffi::custom_newtype!` macro, designed for custom types which use the
"new type" idiom. You still need to specify the type name and builtin type, but because UniFFI
is able to make assumptions about how the type is laid out, `UniffiCustomTypeConverter`
is implemented automatically.

```rust
uniffi::custom_newtype!(NewTypeHandle, i64);
pub struct NewtypeHandle(i64);
```

and that's it!

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

## The `#[uniffi::export(callback_interface)]` attribute

`#[uniffi::export(callback_interface)]` can be used to export a [callback interface](../udl/callback_interfaces.md) definition.
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

## Types from dependent crates

When using proc-macros, you can use types from dependent crates in your exported library, as long as
the dependent crate annotates the type with one of the UniFFI derives.  However, there are a couple
exceptions:

### Types from UDL-based dependent crates

If the dependent crate uses a UDL file to define their types, then you must invoke one of the
`uniffi::use_udl_*!` macros, for example:

```rust
uniffi::use_udl_record!(dependent_crate, RecordType);
uniffi::use_udl_enum!(dependent_crate, EnumType);
uniffi::use_udl_error!(dependent_crate, ErrorType);
uniffi::use_udl_object!(dependent_crate, ObjectType);
```

### Non-UniFFI types from dependent crates

If the dependent crate doesn't define the type in a UDL file or use one of the UniFFI derive macros,
then it's currently not possible to use them in an proc-macro exported interface.  However, we hope
to fix this limitation soon.

## Other limitations

In addition to the per-item limitations of the macros presented above, there is also currently a
global restriction: You can only use the proc-macros inside a crate whose name is the same as the
namespace in your UDL file. This restriction will be lifted in the future.
