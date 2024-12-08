# The `uniffi::Object` derive

The `Object` derive registers a [UniFFI interface](../types/interfaces.md).

```rust
#[derive(uniffi::Object)]
struct MyObject {
    // ...
}
```

The `#[uniffi::export]` attribute is used on an impl block to export methods of the interface.

```rust
#[uniffi::export]
impl MyObject {
    // Constructors need to be annotated as such.
    // The return value can be either `Self` or `Arc<Self>`
    // It is the primary constructor, so in most languages this is invoked with
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
```

Impl blocks without the `#[uniffi::export]` are ignored by UniFFI.
You can use the `#[uniffi::export]` attribute on any number of impl blocks.

See [more about constructors here](./functions.md)

# Traits

```rust
#[uniffi::export]
trait MyTrait {
    // ...
}
```

And a foreign trait:

```rust
#[uniffi::export(with_foreign)]
trait MyTrait {
    // ...
}

```

# Arguments

```rust
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

## Structs implementing traits.

You can declare that an object implements a trait. For example:

```rust
#[uniffi::export]
trait MyTrait { .. }

#[derive(uniffi::Object)]
struct MyObject {}

#[uniffi::export]
impl MyObject {
    // ... some methods
}

#[uniffi::export]
impl MyTrait for MyObject {
    // ... the trait methods.
}
```

This will mean the bindings are able to use both the methods declared directly on `MyObject`
but also be able to be used when a `MyTrait` is required.

Not all bindings support this.

