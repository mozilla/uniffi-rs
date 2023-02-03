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
 - The instance constructor will create an instance of the corresponding `TodoList` Rust struct
 - The owned value is wrapped it in an `Arc<>`
 - The `Arc<>` is lowered into the foreign code using `Arc::into_raw` and returned as an object pointer.

This is the "arc to pointer" dance. Note that this has "leaked" the `Arc<>`
reference out of Rusts ownership system and given it to the foreign-language
code. The foreign-language code must pass that pointer back into Rust in order
to free it, or our instance will leak.

When invoking a method on the instance:
 - The foreign-language code passes the raw pointer back to the Rust code, conceptually passing a "borrow" of the `Arc<>` to the Rust scaffolding.
 - The Rust side calls `Arc::from_raw` to convert the pointer into an an `Arc<>`
 - It wraps the `Arc` in `std::mem::ManuallyDrop<>`, which we never actually
   drop.  This is because the Rust side is are borrowing the Arc and shouldn't
   run the destructor and decrement the reference count.
 - The `Arc<>` is cloned and returned to the Rust code

Finally, when the foreign-language code frees the instance, it
passes the raw pointer a special destructor function so that the Rust code can
drop that initial reference (and if that happens to be the final reference,
the Rust object will be dropped.).  This simply calls `Arc::from_raw`, then
lets the value drop.

Passing instances as arguments and returning them as values works similarly, except that
UniFFI does not automatically wrap/unwrap the containing `Arc`.

To see this in action, use `cargo expand` to see the exact generated code.
