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
the corresponding type. However, you can add a `[Self=ByArc]` attribute to constructors if
your constructor already returns an `Arc` and UniFFI will use that `Arc` directly.

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

On the Rust side of the generated bindings, the instance constructor will create an instance of the
corresponding `TodoList` Rust struct, wrap it in an `Arc<>` and return the Arc's raw pointer to the
foreign language code:

```rust
pub extern "C" fn todolist_12ba_TodoList_new(
    err: &mut uniffi::deps::ffi_support::ExternError,
) -> *const std::os::raw::c_void /* *const TodoList */ {
    uniffi::deps::ffi_support::call_with_output(err, || {
        let _new = TodoList::new();
        let _arc = std::sync::Arc::new(_new);
        <std::sync::Arc<TodoList> as uniffi::FfiConverter>::lower(_arc)
    })
}
```

The UniFFI runtime implements lowering for object instances using `Arc::into_raw`:

```rust
unsafe impl<T: Sync + Send> FfiConverter for std::sync::Arc<T> {
    type FfiType = *const std::os::raw::c_void;
    fn lower(self) -> Self::FfiType {
        std::sync::Arc::into_raw(self) as Self::FfiType
    }
}
```

which does the "arc to pointer" dance for us. Note that this has "leaked" the
`Arc<>` reference out of Rusts ownership system and given it to the foreign-language code.
The foreign-language code must pass that pointer back into Rust in order to free it,
or our instance will leak.

When invoking a method on the instance, the foreign-language code passes the
raw pointer back to the Rust code, conceptually passing a "borrow" of the `Arc<>` to
the Rust scaffolding. The Rust side turns it back into a cloned `Arc<>` which
lives for the duration of the method call:

```rust
pub extern "C" fn todolist_12ba_TodoList_add_item(
    ptr: *const std::os::raw::c_void,
    todo: uniffi::RustBuffer,
    err: &mut uniffi::deps::ffi_support::ExternError,
) -> () {
    uniffi::deps::ffi_support::call_with_result(err, || -> Result<_, TodoError> {
        let _retval = TodoList::add_item(
          &<std::sync::Arc<TodoList> as uniffi::FfiConverter>::try_lift(ptr).unwrap(),
          <String as uniffi::FfiConverter>::try_lift(todo).unwrap())?,
        )
        Ok(_retval)
    })
}
```

The UniFFI runtime implements lifting for object instances using `Arc::from_raw`:

```rust
unsafe impl<T: Sync + Send> FfiConverter for std::sync::Arc<T> {
    type FfiType = *const std::os::raw::c_void;
    fn try_lift(v: Self::FfiType) -> Result<Self> {
        let v = v as *const T;
        // We musn't drop the `Arc<T>` that is owned by the foreign-language code.
        let foreign_arc = std::mem::ManuallyDrop::new(unsafe { Self::from_raw(v) });
        // Take a clone for our own use.
        Ok(std::sync::Arc::clone(&*foreign_arc))
    }
```

Notice that we take care to ensure the reference that is owned by the foreign-language
code remains alive.

Finally, when the foreign-language code frees the instance, it
passes the raw pointer a special destructor function so that the Rust code can
drop that initial reference (and if that happens to be the final reference,
the Rust object will be dropped.)

```rust
pub extern "C" fn ffi_todolist_12ba_TodoList_object_free(ptr: *const std::os::raw::c_void) {
    if let Err(e) = std::panic::catch_unwind(|| {
        assert!(!ptr.is_null());
        unsafe { std::sync::Arc::from_raw(ptr as *const TodoList) };
    }) {
        uniffi::deps::log::error!("ffi_todolist_12ba_TodoList_object_free panicked: {:?}", e);
    }
}
```

Passing instances as arguments and returning them as values works similarly, except that
UniFFI does not automatically wrap/unwrap the containing `Arc`.
