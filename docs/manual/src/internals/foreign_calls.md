# Rust -> Foreign calls

How can Rust code make a call into the foreign language?
The [lifting and lowering](./lifting_and_lowering.md) docs describe this at a high-level, this document will explore the details.

This document only describes non-async calls.
See the [Async FFI](./async-ffi.md) docs for a description of async calls.

## VTables

Calling into foreign code has 2 major differences from [calling Rust code](./rust_calls.md):

* Foreign code is only called via an interface.
* The Rust code does not have knowledge of the FFI functions on the foreign language side.

Because of this, the foreign calls all go through a vtable defined in the foreign language.
This is a `repr(C)` struct where each entry is a function pointer corresponding to a method in a callback interface.

For example:

```rust
/// For this trait interface...
#[uniffi::export(with_foreign)]
pub trait TodoList {
    fn append(&self, title: String);
    fn get_items(&self) -> Vec<String>;
}

/// A VTable like this will be generated:
#[repr(C)]
pub struct UniffiTodoListVTable {
    // One function pointer field for each method
    // title is passed as a `RustBuffer` as described in [Lifting and lowering](./lifting_and_lowering.md)
    append: extern "C" fn(uniffi_handle: u64, title: RustBuffer, uniffi_out_return: &mut (), uniffi_call_status: &mut RustCallStatus),
    get_items: extern "C" fn(uniffi_handle: u64, uniffi_out_return: &mut RustBuffer, uniffi_call_status: &mut RustCallStatus);
    // A function pointer for freeing the callback interface
    uniffi_free: extern "C" fn(uniffi_handle: u64);
}

/// Called by the generated foreign bindings to register the VTable
#[no_mangle]
pub extern "C" fn uniffi_init_todo_list_vtable(vtable: std::ptr::NonNull<UniffiTodoListVTable>) {...}
```


## VTable methods

VTable methods work similar to a [Rust method call](./rust_calls.md)], with some exceptions.

Instead of returning the value as normal, it's written to a pointer that Rust passes as extra argument to the method (AKA an out pointer).
This is because some foreign languages, like Python, don't support returning C structs like `RustBuffer` ([issue](https://github.com/python/cpython/issues/49960)).
Methods that don't return anything still have an out pointer argument, but it's never written to.

The first argument is a `u64` handle.
Different bindings have different systems for callback interface handles, but they generally fall into 2 camps:
* The handle is the key to a hash map that maps `u64` -> callback interface objects.
* The is a raw pointer to a callback interface object cast to a `u64`.

The method call is handled very similarly to a Rust method call:

* If the function returns successfully then the lowered return value is written to `uniffi_out_return` and nothing is written to `uniffi_call_status`.
* If the function throws the exception that corresponds to the `Result::Err` side of the Rust function, then:
  * The foreign bindings set `uniffi_call_status.code` to `RustCallStatusCode::Error`
  * The foreign bindings serialize the error into a `RustBuffer` and write it to `uniffi_call_status.error_buf`.
  * Nothing is written to `uniffi_out_return`.
* If the method throws some other exception, then:
  * The foreign bindings set `uniffi_call_status.code` to `RustCallStatusCode::InternalError`
  * The foreign bindings should try to to serialize the error message to a `RustBuffer` and write it to `error_buf`.
    However, bindings may also not write to `error_buf` and the generated Rust code will handle this.
  * Nothing is written to `uniffi_out_return`.
  * If the callback method returns a `Result<T, E>` type and the bindings implement `From<uniffi::UnexpectedUniFFICallbackError> for E`,
    then the generated Rust will convert the error to an `Err(E)` value and return it.
  * If not, then the Rust bindings will panic.
    This is why it's recommended to always have callback methods return `Result` types and implement `From<uniffi::UnexpectedUniFFICallbackError>`.

## VTable free method

Each vtable also stores a free method that inputs a `uniffi_handle`.
The generated Rust code calls this in the `Drop` implementation for the callback interface.
The foreign bindings are responsible for releasing any resources associated with the handle, this usually means removing the hash map entry for it or freeing the raw pointer.

## Registering VTables and calling methods

As shown in the example code, UniFFI generates a function to register the vtable for each callback interface.
The foreign bindings must all this function before returning any callback interface handles, usually this is done at startup.
When a callback interface method is called, the generated code finds the registered vtable, looks up the field for the method, then uses the function pointer for that field to make the call.
