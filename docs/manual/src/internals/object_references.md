# Managing Object References

Uniffi [interfaces](../udl/interfaces.md) represent instances of objects
that have methods and contain state. One of Rust's core innovations
is its ability to provide compile-time guarantees about working with such instances,
including:

* Ensuring that each instance has a unique owner responsible for disposing of it.
* Ensuring that there is only a single writer *or* multiple readers of an object
  active at any point in the program.
* Guarding against data races.

The very nature of the problems Uniffi tries to solve is that calls may come
from foreign languages on any thread. Uniffi itself tries to take a hands-off
approach as much as possible to allow the Rust compiler itself to ensure these
guarantees can be met - which in practice means all instances exposed by uniffi
must be, in Rust terminology, `Send+Sync`, and `&mut self` type params
typically can't be supported.

Typically this will mean your implementation uses some data structures
explicitly designed for this purpose, such as a `Mutex` or `RwLock` - but this
detail is completely up to you - as much as possible, uniffi tries to stay out
of your way, so ultimately it is the Rust compiler itself which is the ultimate
arbiter.

## Arcs

In order to allow for instances to be used as flexibly as possible, uniffi
works with `Arc`s holding a pointer to your instances and leverages their
reference-count based lifetimes, allowing uniffi to largely stay out
of handling lifetimes entirely for these objects.

However, this does come at a cost - when you want to return instances from
your dictionaries or methods, you must return an `Arc<>` directly. When
accepting instances as arguments, you can choose to accept it as an `Arc<>` or
as the underlying struct - there are different use-cases for each scenario.

The exception to the above is constructors - these are expected to just provide
the instance and uniffi will wrap it in the `Arc<>`.

For example, given a interface definition like this:

```idl
interface TodoList {
    constructor();
    void add_item(string todo);
    sequence<string> get_items();
};
```

On the Rust side of the generated bindings, the instance constructor will create an instance of the
corresponding `TodoList` Rust struct, wrap it in an `Arc<>` and return a raw
pointer to the foreign language code:

```rust
pub extern "C" fn todolist_12ba_TodoList_new(
    err: &mut uniffi::deps::ffi_support::ExternError,
) -> *const std::os::raw::c_void /* *const TodoList */ {
    uniffi::deps::ffi_support::call_with_output(err, || {
        let _new = TodoList::new();
        let _arc = std::sync::Arc::new(_new);
        uniffi::UniffiVoidPtr(<std::sync::Arc<TodoList> as uniffi::ViaFfi>::lower(_arc))
    })
}
```

and the uniffi runtime defines:

```rust
unsafe impl<T: Sync + Send> ViaFfi for std::sync::Arc<T> {
    type FfiType = *const std::os::raw::c_void;
    fn lower(self) -> Self::FfiType {
        std::sync::Arc::into_raw(self) as Self::FfiType
    }
}
```

which does the "arc to pointer" dance for us. Note that this has "leaked" the
`Arc<>` reference - if we never see that pointer again, our instance will leak.

When invoking a method on the instance, the foreign-language code passes the
raw pointer back to the Rust code, which turns it back into a cloned `Arc<>` which
lives for the duration of the method call:

```rust
pub extern "C" fn todolist_12ba_TodoList_add_item(
    ptr: *const std::os::raw::c_void,
    todo: uniffi::RustBuffer,
    err: &mut uniffi::deps::ffi_support::ExternError,
) -> () {
    uniffi::deps::ffi_support::call_with_result(err, || -> Result<_, TodoError> {
        let _obj = <std::sync::Arc<TodoList> as uniffi::ViaFfi>::try_lift(ptr).unwrap();
        let _retval =
            TodoList::add_item(&_obj, <String as uniffi::ViaFfi>::try_lift(todo).unwrap())?;
        Ok(_retval)
    })
}
```

where the uniffi runtime defines:

```rust
unsafe impl<T: Sync + Send> ViaFfi for std::sync::Arc<T> {
    type FfiType = *const std::os::raw::c_void;
    fn try_lift(v: Self::FfiType) -> Result<Self> {
        let v = v as *const T;
        // We musn't drop the `Arc<T>` that is owned by the foreign-language code.
        let foreign_arc = std::mem::ManuallyDrop::new(unsafe { Self::from_raw(v) });
        // Take a clone for our own use.
        Ok(std::sync::Arc::clone(&*foreign_arc))
    }
```

Notice that we take care to ensure the reference added by the constructor
remains alive. Finally, when the foreign-language code frees the instance, it
passes the raw pointer a special destructor function so that the Rust code can
drop that initial final reference (and if that happens to be the final reference,
the rust object will be dropped.)

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

## Managing Concurrency

You might be noticing a distinct lack of concurrency management, and this is
by design - it means that concurrency management is the responsibility of the
Rust implementations. The `T` in an `Arc<T>` is supplied by the Rust code
being wrapped and the Rust compiler will complain if that isn't `Send+Sync`.
This means that uniffi can take a hands-off approach, letting the Rust compiler
guide the component author.