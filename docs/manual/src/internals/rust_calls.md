# Foreign -> Rust calls

What happens when code in the foreign language makes a call to Rust?
The [lifting and lowering](./lifting_and_lowering.md) docs describe this at a high-level, this document will explore the details.

This document only describes non-async Rust calls.
See the [Async FFI](./async-ffi.md) docs for a description of async calls.

## The FFI function

For each exported Rust function, an FFI function is generated.
This `extern "C"` function is exported in the Rust library and its what's called by the foreign code.

Different bindings use different techniques to call these functions.
For example, Python uses [ctypes](https://docs.python.org/3/library/ctypes.html) to call into a dynamic library, while Swift links to a static library and calls the C functions directly.

The signature for the FFI function is:
* Arguments:
  * The [lowered type](./lifting_and_lowering.md) for each argument of the Rust function
  * A `RustCallStatus` out pointer
* Returns:
  * For non-unit type returns: the [lowered type](./lifting_and_lowering.md) of the return value for the Rust function
  * For unit type returns: `void` (i.e. no return value, which is special-cased in `C`).

## RustCallStatus

The last argument for the FFI function is a pointer to a `RustCallStatus` struct.
The FFI function writes to this struct to report errors when calling the Rust function.

Here's the layout for this struct:

```rust
#[repr(C)]
pub struct RustCallStatus {
    pub code: RustCallStatusCode,
    pub error_buf: RustBuffer,
}

#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RustCallStatusCode {
    Success = 0,
    Error = 1,
    UnexpectedError = 2,
    Cancelled = 3, // Only used for async calls
}
```

When the foreign code calls the Rust FFI function, it initializes `RustCallStatus.code` to `RustCallStatus::Success` and `RustCallStatus.error_buf` to an empty RustBuffer.  After calling the real Rust function, the generated FFI function writes to this struct:

* If the function returns successfully (i.e. it runs normally and returns `Ok` value for `Result` return types), then nothing is written.
  Since the foreign bindings initialize `code=RustCallStatus::Success`, this will be interpreted as a successful call.
* If a `Result::Err` value is returned then:
  * The generated Rust code sets `code` to `RustCallStatusCode::Error`
  * The generated Rust code serializes the error into a `RustBuffer` and stores that in `error_buf`.
  * The foreign bindings are responsible for deserializing that error into an exception value and
    throwing that exception.
  * A placeholder value is returned (e.g. `0` or an empty `RustBuffer`);
* If an unexpected error happens, for example a failure when lifting the arguments, then:
  * The generated Rust code sets `code` to `RustCallStatusCode::InternalError`
  * The generated Rust code tries to serialize the error message to a `RustBuffer` and store it `error_buf`.
    However, it's possible this fails in which case `error_buf` not be written to.
  * A placeholder value is returned
  * The foreign bindings are responsible throwing some sort of UniFFI internal exception.
    If possible, this exception should contain the error message.

## Method calls

Method calls work the same as function calls, except there's an extra argument for the object handle.
This means the arguments are:

* The object handle (`void *`).
  This is cloned before making the call as described in [object references](./object_references.md).
* The lowered type for each Rust argument
* A `RustCallStatus` out pointer

## Panic handling

UniFFI uses `std::panic::catch_unwind` to try to catch any panics in the Rust code.
The main reason is to prevent them from moving up the call stack into the foreign language's frames, which would almost certainly lead to a crash.
Panics are treated as unexpected errors as described above.
UniFFI can't always catch panics, for example when the Rust panic handler is set to `abort`.
