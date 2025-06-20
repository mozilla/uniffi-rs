# Managing Object References

UniFFI [interfaces](../udl/interfaces.md) represent instances of objects
that have methods and contain state. One of Rust's core innovations
is its ability to provide compile-time guarantees about working with such instances,
including:

* Ensuring that each instance has a unique owner responsible for disposing of it.
* Ensuring that there is only a single writer *or* multiple readers of an object
  active at any point in the program.
* Guarding against data races.

The very nature of the problems UniFFI tries to solve is that calls may come
from foreign languages on any thread, outside of the control of Rust's ownership
system. UniFFI itself tries to take a hands-off approach as much as possible and
depends on the Rust compiler itself to uphold safety guarantees, without assuming
that foreign-language callers will be "well behaved".

## Concurrency

UniFFI's hands-off approach means that all object instances exposed by UniFFI must be safe to
access concurrently. In Rust terminology, they must be `Send+Sync` and must be useable
without taking any `&mut` references.

Typically this will mean that the Rust implementation of an object uses some of Rust's
data structures for thread-safe interior mutability, such as a `Mutex` or `RwLock` or
the types from `std::atomic`. The precise details are completely up to the author
of the component - as much as possible, UniFFI tries to stay out of your way, simply requiring
that the object implementation is `Send+Sync` and letting the Rust compiler ensure that
this is so.

## Lowered representation

Interfaces are lowered as `u64` handles.
`0` is reserved as an invalid value.

## Lifetimes

In order to allow for instances to be used as flexibly as possible from foreign-language code,
UniFFI wraps all object instances in an `Arc` and leverages their reference-count based lifetimes,
allowing UniFFI to largely stay out of handling lifetimes entirely for these objects.

When constructing a new object, UniFFI is able to add the `Arc` automatically, because it
knows that the return type of the Rust constructor must be a new uniquely-owned struct of
the corresponding type.

When you want to return object instances from functions or methods, or store object instances
as fields in records, the underlying Rust code will need to work with `Arc<T>` directly, to ensure
that the code behaves in the way that UniFFI expects.

When accepting instances as arguments, the underlying Rust code can choose to accept it as an `Arc<T>`
or as the underlying struct `T`, as there are different use-cases for each scenario.

For example, given a interface definition like this:

```idl
interface TodoList {
    constructor();
    void add_item(string todo);
    sequence<string> get_items();
};
```

On the Rust side of the generated bindings:
* The instance constructor will create an instance of the corresponding `TodoList` Rust struct
* The owned value is wrapped in an `Arc<>`
* The `Arc<>` is lowered into the foreign code using `Arc::into_raw`, casted to a `u64` and returned

This is the "arc to pointer" dance. Note that this has "leaked" the `Arc<>`
reference out of Rusts ownership system and given it to the foreign-language
code. The foreign-language code must pass that pointer back into Rust in order
to free it, or our instance will leak.

## Cloning handles and calling methods

The generate Rust code defines a clone function for each interface that will clone the handle.
This FFI function:

* Inputs a `u64` value.
* Casts the `u64` to a `*T` and calls [Arc::increment_strong_count](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.increment_strong_count)
* Returns the `u64` back.
* The foreign side can now treat both `u64` values as separate handles with their own lifetimes.
  This is safe, since we've effectively leaked 2 `Arc<>`s.

This is used by the foreign-language code to invoke instance methods:

* Clone the handle
* Pass the handle to the Rust FFI function, conceptually transferring ownership of a leaked `Arc<>` back to Rust.
* The Rust side casts the handle to a `*T` then calls `Arc::from_raw` to reconstruct the `Arc<>`

## Passing handles across the FFI

Passing instances as arguments and returning them as values works similarly to calling methods.
The foreign language code clones the handle, then returns the cloned value.

## Freeing handles

The Rust code defines a free function for each interface that frees a handle.
This FFI function:

* Inputs a `u64` value.
* Casts the `u64` to a `*T` and calls [Arc::decrement_strong_count](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.increment_strong_count)

When the foreign-language code is ready to destroy an object, calls this free function transferring ownership of the `Arc<>` back to Rust.
The generated Rust code simply calls `Arc::from_raw`, then lets the value drop.

## Callback interfaces

Callback interfaces are interfaces defined on the foreign side of things.
Conceptually these work the same as regular interfaces:

* The foreign side defines a free and clone function in the [vtable](./foreign_calls.md).
* Passing a callback interface handle across the FFI represents transferring ownership.

However, the mechanics of managing references differ from Rust.
Foreign languages are free to implement them however they want, but they usually use a handle map:

* Handle maps map `u64` values to object instances.
* To create/clone a new handle, insert a new entry and return the key.
* To free a handle, remove the entry for the key.

## Trait interfaces

Trait interfaces are used to pass `Arc<dyn Trait>` instances across the FFI.
This presents an extra difficulty since `dyn Trait` is unsized and `*dyn Trait` is a wide pointer that takes 2 words to store.

To work around this, UniFFI adds an extra Arc creating a `Arc<Arc<dyn Trait>>` value.
This effectively converts the wide pointer into a regular pointer, since `Arc<dyn Trait>` is sized (at 2 words).

## Trait interfaces with foreign support

These are trait interfaces that can also be implemented on the foreign side.
These are effectively hybrids of callback interfaces and trait interfaces:

* They are passed from the foreign language to Rust as a callback interface
* They are passed from Rust to the foreign language as a trait interface.
