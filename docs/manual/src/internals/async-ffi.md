# UniFFI Async FFI details

This document describes the low-level FFI details of UniFFI async calls.
Check out [Async overview](./async-overview.md) for high-level description of what's going on
here.

## Rust async functions

Rust async functions are implemented by a [scaffolding](../glossary.md#scaffolding) function that return a `RustFuture` handle.
For example, a `fn add(a: u32, b: u32) -> u32` function would be implemented by something like this:

```rust
pub extern "C" fn ffi_add(a: u32, b: u32) -> uniffi::Handle {
    // Creates a future for the `add` function and returns handle that represents that handle.
}
```

The [bindings](../glossary.md#bindings) then sets up an asynchronous loop that polls the RustFuture until it's complete.

Here's some Python code that shows how this works:

```python
async def add(a, b):
    # Get a future handle by calling the scaffolding function
    rust_future_handle = RustScaffoldingLibrary.ffi_add(a, b)
    try:
        # asynchronously loop until the future is ready
        while True:
            # create a future to handle a single iteration of the polling loop.
            inner_future = eventloop.create_future()
            # Create a handle to send to Rust that represents `inner_future`
            inner_future_handle = create_future_handle(inner_future)
            # Call the rust_future_poll scaffolding function (see below for how that's handled).
            RustScaffoldingLibrary.ffi_rust_future_poll_u32(
                rust_future_handle,
                uniffi_continuation_callback,
                inner_future_handle,
            )
            # await the inner future.  When this completes, there's are 2 possibilities:
            #  * The RustFuture is ready and we can complete it
            #  * The RustFuture should be polled again and we should perform another iteration
            poll_code = await inner_future
            if poll_code == _UNIFFI_RUST_FUTURE_POLL_READY:
                break

        # Call the complete function to get:
        #   * The future's return value
        #   * The call status -- returned by an out pointer and used to return errors.
        call_status = UniffiRustCallStatus.default()
        return_value = RustScaffoldingLibrary.rust_future_complete_u32(rust_future_handle, ctypes.byref(call_status))

        # We can then lift the result the same way we would a sync call
        return lift_result(return_value, call_status)
    finally:
        # In the finally block, we call `rust_future_free` to ensure we cleanup the future handle
        # regardless of any errors
        RustScaffoldingLibrary.rust_future_free_u32(rust_future_handle)

# Continuation callback, this is called from Rust when progress can be made on the future, either
# because it's ready or it needs to be polled again.
def uniffi_continuation_callback(future_handle, poll_code):
    # Convert the handle we sent to Rust to a Python Future
    future = get_future_from_handle(future_handle)
    # Complete the future with the code that indicates if the future is ready or not.
    eventloop.call_soon_threadsafe(future.set_result, poll_code)

# Create a `u64` handle that represents a Python Future object.
def create_future_handle(future):
    # The code to do this will vary by language.

# Re-create the Python Future object from a `u64` handle.
def get_future_from_handle(handle):
    # The code to do this will vary by language.
```

### Cancellation

Some languages have builtin cancellation semantics.  For those, you can call the
`rust_future_cancel` to request that the future be cancelled.  On the Rust side, this causes the
future to be dropped.

```python
def cancel_future(future_handle):
    if rust_future_has_not_been_freed(future_handle):
        RustScaffoldingLibrary.rust_future_cancel_u32(rust_future_handle)
```

### Managing the RustFuture handle

* The foreign code is responsible for calling the `rust_future_free` function when it's done with the handle.
  The code must always call `rust_future_free`, regardless of if the future is completed or cancelled.
  Once that function is called, the handle must not be used again.
* `rust_future_complete` must only be called once.  Once it's called, `rust_future_poll` should not
  be called again.
* If your bindings call `rust_future_cancel` make sure there are no races that allow it to be called
  after `rust_future_free`.

### FFI definitions

#### Scaffolding functions

```rust
extern "C" fn [scaffolding_function_name](
    // The lowered type for each argument of the Rust function
    foo: u32,
    bar: RustBuffer,
    // No `RustCallStatus` argument.  Errors are handled by calling the `rust_future_complete`
    // function.
) -> u64 { } // Returns a RustFuture handle
```

#### rust_future_poll

A `rust_future_poll` method is defined for each lowered return type.  For example:
```rust
/// `rust_future_poll` for return types that lower to `RustBuffer`.  This handles exported functions
/// that return `String`, `Vec`, records, etc.
extern "C" fn rust_future_poll_rustbuffer(
    rust_future_handle: u64,
    /// Callback to call when progress can be made to the future
    continuation_callback: RustFutureContinuationCallback,
    /// Data to pass to the continuation callback
    continuation_callback_data: u64,
) { }

type RustFutureContinuationCallback = fn(callback_data: u64, poll_code: u8)

// The future is ready, call `rust_future_complete`
const POLL_CODE_RUST_FUTURE_READY = 0;
// Wake the future by calling `rust_future_poll` again.
const POLL_CODE_RUST_FUTURE_WAKE = 1;
```

#### Other RustFuture FFI functions

```rust
/// `rust_future_complete` for return types that lower to `u8`
extern "C" fn rust_future_complete_u8(
    rust_future_handle: u64,
    /// RustCallStatus out pointer.  This indicates if the call was successful.
    *mut RustCallStatus out_status,
) -> u8 { } // lowered return value, in this case a `u8`.

/// `rust_future_cancel` for return types that lower to `f32`.
///
/// Languages that support cancellation can call this to cancel the future.  It will cause the
/// `Future` object to be dropped in Rust.
extern "C" fn rust_future_cancel_f32(
    rust_future_handle: u64,
) { }

/// `rust_future_free` for void return types.
///
/// This must be called for each RustFuture you receive and it must be the last call you pass the
/// RustFuture handle to.
extern "C" fn rust_future_free_void(
    rust_future_handle: u64,
) { }
```

## Foreign async callback interface methods

Async callback interface methods are defined as fields in the callback interface vtable like sync
methods.  However, they have no return value and 3 extra arguments:

- `complete_func`: Function to call when the async method completes.
- `complete_func_data`: `u64` value to pass to the complete function
- `foreign_future_dropped_callback`: Out pointer to the future dropped callback.
  If the foreign bindings set this, it will be called when the Rust future is dropped.
  This is used to handle cancellation for languages that support it.


For example: 

```rust
#[repr(C)]
struct CallbackInterfaceVTable {
    add: extern "C" fn(
        // The lowered type for each argument of the method
        a: u32,
        b: u32,
        /// Function pointer for the completion func
        complete_func: ForeignFutureCompleteU32,
        /// Data to pass to the completion func
        complete_func_data: u64,
        /// Out pointer that can be used to set the future dropped callback
        out_dropped_callback: *mut ForeignFutureDroppedCallbackStruct,
    ), // Note: no return value, `complete_func` is used for that.
    // .. other methods here
}

/// Complete func signature, details in the next section
type ForeignFutureCompleteU32 = extern "C" fn(u64, ForeignFutureResultU32);
```

### Completing async methods with `complete_func`

The `complete_func` should be called when the async callback method has completed and ready to
return data.  Pass it 2 arguments:

- The `complete_func_data` passed to the callback method.
- A foreign future result struct, which contains the return value and the RustCallStatus for the
  call.

The foreign future result struct varies based on the return value:

```rust
#[repr(C)]
/// Result struct for `u8` 
struct ForeignFutureResultU8 {
    /// Lowered return value.  For error calls, set this to a placeholder value like `0`
    return_value: u8,
    /// RustCallStatus.  This indicates if the call was successful or not
    call_status: RustCallStatus,
}

/// Result struct for `RustBuffer` 
struct ForeignFutureResultRustBuffer {
    /// Lowered return value.  For error calls, set this to empty `RustBuffer`.
    return_value: RustBuffer,
    /// RustCallStatus.  This indicates if the call was successful or not
    call_status: RustCallStatus,
}

/// Result struct for void returns
struct ForeignFutureResultVoid {
    // No return value field.

    /// RustCallStatus.  This indicates if the call was successful or not
    call_status: RustCallStatus,
}

/// ...etc.
```

### Cancellation with `foreign_future_dropped_callback`

The `foreign_future_dropped_callback` is a pointer to a `ForeignFutureDroppedCallbackStruct`.
This can be used to get a callback when the underlying Future is dropped in `Rust`.
The main reason to do this is to cancel the Foreign async task.  If you want to support
cancellation, set the `ForeignFutureDroppedCallbackStruct` data and connect it to some mechanism for
cancelling the future.

```
type ForeignFutureDroppedCallback = extern "C" fn(u64);

#[repr(C)]
struct ForeignFutureDroppedCallbackStruct {
    /// Data to pass to the callback
    callback_data: u64,
    /// Callback function pointer
    callback: ForeignFutureDroppedCallback,
}
```

Languages that don't want to support cancellation are free to ignore this field by not setting a value.

### Managing the ForeignFuture handle

If an async callback method is called, you must ensure that the `complete_func` is called which will
complete the future and allow the async task to progress.  The `complete_func` must be called
exactly once.

If you don't want to support cancellation, then managing this handle is fairly easy:

* Start an async task to execute the method
* Chain a function to that task that calls `complete_func` with the handle when it's complete
* Ensure that `complete_func` is also called if the function errors out.
  In this case you can pass a `ForeignFutureResult*` value with
  `RustCallStatus.code = CallStatus::InternalError`.

If you want to support cancellation, then there are some additional steps:

* Create a `u64` handle for the async task that was started.
* Set the `foreign_future_dropped_callback` with a callback and handle.
* In the callback:
   * Cancel the task if it's still running
   * Release the handle in whatever way the foreign language requires
   * Note: this callback can be called after the task completes. You may need to avoid calling
     methods that are invalid for completed tasks.
