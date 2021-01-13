# Managing Object References

Uniffi [interfaces](../udl/interfaces.md) represent instances of objects
that have methods and contain shared mutable state. One of Rust's core innovations
is its ability to provide compile-time guarantees about working with such instances,
including:

* Ensuring that each instance has a unique owner responsible for disposing of it.
* Ensuring that there is only a single writer *or* multiple readers of an object
  active at any point in the program.
* Guarding against data races.

Uniffi aims to maintain these guarantees even when the Rust code is being invoked
from a foreign language, at the cost of turning them into run-time checks rather
than compile-time guarantees.

We achieve this by indirecting all object access through a
[handle map](https://docs.rs/ffi-support/0.4.0/ffi_support/handle_map/index.html),
a mapping from opaque integer handles to object instances. This indirection
imposes a small runtime cost but helps us guard against errors or oversights
in the generated bindings.

For each interface declared in the UDL, the uniffi-generated Rust scaffolding
will create a global [ffi_support::ConcurrentHandleMap](https://docs.rs/ffi-support/0.4.0/ffi_support/handle_map/struct.ConcurrentHandleMap.html) that is responsible for owning all instances
of that interface, and handing out references to them when methods are called.

For example, given a interface definition like this:

```idl
interface TodoList {
    constructor();
    void add_item(string todo);
    sequence<string> get_items();
};
```

The Rust scaffolding would define a lazyily-initialized global static like:

```rust
lazy_static! {
    static ref UNIFFI_HANDLE_MAP_TODOLIST: ConcurrentHandleMap<TodoList> = ConcurrentHandleMap::new();
}
```

On the Rust side of the generated bindings, the instance constructor will create an instance of the
corresponding `TodoList` Rust struct, insert it into the handlemap, and return the resulting integer
handle to the foreign language code:

```rust
pub extern "C" fn todolist_TodoList_new(err: &mut ExternError) -> u64 {
    // Give ownership of the new instance to the handlemap.
    // We will only ever operate on borrowed references to it.
    UNIFFI_HANDLE_MAP_TODOLIST.insert_with_output(err, || TodoList::new())
}
```

When invoking a method on the instance, the foreign-language code passes the integer handle back
to the Rust code, which borrows a reference to the instance from the handlemap for the duration
of the method call:

```rust
pub extern "C" fn todolist_TodoList_add_item(handle: u64, todo: RustBuffer, err: &mut ExternError) -> () {
    let todo = <String as uniffi::ViaFfi>::try_lift(todo).unwrap()
    // Borrow a reference to the instance so that we can call a method on it.
    UNIFFI_HANDLE_MAP_TODOLIST.call_with_result_mut(err, handle, |obj| -> Result<(), TodoError> {
        TodoList::add_item(obj, todo)
    })
}
```

Finally, when the foreign-language code frees the instance, it passes the integer handle to
a special destructor function so that the Rust code can delete it from the handlemap:

```rust
pub extern "C" fn ffi_todolist_TodoList_object_free(handle: u64) {
    UNIFFI_HANDLE_MAP_TODOLIST.delete_u64(handle);
}
```

This indirection gives us some important safety properties:

* If the generated bindings incorrectly pass an invalid handle, or a handle for a different type of object,
  then the handlemap will throw an error with high probability, providing some amount of run-time typechecking
  for correctness of the generated bindings.
* The `ConcurrentHandleMap` class wraps each instance with a `Mutex`, which serializes access to the instance
  and upholds Rust's guarantees against shared mutable access.