# Exposing standard Rust traits

Rust has a number of general purpose traits which add functionality to types, such
as `Debug`, `Display`, etc. It's possible to tell UniFFI that your type implements these
traits and to generate FFI functions to expose them to consumers. Bindings may then optionally
generate special methods on the object.

The list of supported traits is hard-coded in UniFFI's internals, and at time of writing
is `Debug`, `Display`, `Eq`, `Ord` and `Hash`.

This is supported primarily for Interfaces - it's also supported for Records and Enums, but it has fewer use-cases there - see [data-classes](#data-classes) below.

For example, in proc-macros:
```rust
#[derive(Debug, uniffi::Object)]
#[uniffi::export(Debug)]
struct TodoList {
   ...
}
```
or in UDL
```
[Traits=(Debug)]
interface TodoList {
    ...
};
```
with the matching Rust:
```rust
#[derive(Debug)]
struct TodoList {
   ...
}
```

The bindings will generate:

* Python: The object will have a `__repr__` method that returns the value implemented by the `Debug` trait.
* Swift: Will generate a `public var debugDescription: String {..}` method on the object, allowing, eg, `String(reflecting: ob)` to be used.
* Kotlin: Will generate `override fun toString(): String {..}` (although will prefer the `Display` implementation if it exists).

Whereas `Eq` would generate:

* Python: The object will have `__eq__(self, other)` and `__ne__(self, other)` methods.
* Swift: Will generate `public static func == (self: .., other: ..)`
* Kotlin: Will generate `override fun equals(other: Any?): Boolean` and have the object extend `Comparable<>`.

etc.

External bindings may not support these, so they might be ignored.

It is your responsibility to implement the trait on your objects; UniFFI will attempt to generate a meaningful error if you do not.

# Data-classes

This is supported for `Record`s and `Enum`s - but take care - every time one of the generated methods is called,
the entire `Record` or `Enum` will be copied by-value across the FFI.
This isn't a problem for `Interface`s because exactly one pointer is copied, but data classes serialize and deserialize the entire object, which is expensive for large objects.

Further, UniFFI will automatically generate some of this support when these traits are not specified.
For example, all bindings will automatically generate simple equality check and hash functions for records which compare/hash each element in the record.
However, if you explicitly export the `Eq` trait, these builtin implementations will not be generated and instead will be replaced with a significantly more expensive copy and call into Rust code.
So if your `Eq` implementation is `#[derive()]`d, you are probably just providing the same semantics with a much slower implementation.

It is expected this support will prove most useful for the `Debug` and `Display` traits and used primarily for debugging,
but it's there if you need it for obscure cases where you really do need custom functionality and the cost is ok.

```rust
#[derive(uniffi::Record, Debug)]
#[uniffi::export(Debug, Eq)]
// Your `Eq` impl - there's no advantage exposing a derived `Eq`
impl Eq for TraitRecord { .. }
pub struct TraitRecord { .. }
```
