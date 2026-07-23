# Byte buffers (`&[u8]`)

A function argument typed `&[u8]` (or `[ByRef] bytes` in UDL) lets
Rust *borrow* a foreign-owned byte buffer for the duration of the
FFI call — no copy is made. This is useful for large payloads
(images, network frames, audio buffers) that would otherwise be
duplicated on every call.

For owned bytes, or for any case the constraints below disallow,
use `Vec<u8>` / `bytes` instead.

## Usage

In Rust, declare the parameter as a borrow in the usual way:

```rust
#[uniffi::export]
pub fn parse_frame(data: &[u8]) -> Frame {
    // `data` borrows the foreign buffer; valid for this call only.
    ...
}
```

In UDL, mark the parameter `[ByRef] bytes`:

```idl
namespace example {
    Frame parse_frame([ByRef] bytes data);
};
```

## Constraints

- **Direction.** `&[u8]` only flows foreign → Rust. A function may
  take it as an argument; it cannot be returned, used in a callback
  result, or used as a trait-method return value.
- **Argument position only.** `&[u8]` cannot be nested inside a
  record, sequence, map, `Option`, or `Result`. The borrow lives
  only as long as the FFI call, so structures that could outlive
  the call are rejected.
- **Lifetime.** The slice is valid only inside the body of the Rust
  function. Don't store it, don't hand it to a spawned task, and
  don't capture it in async work that outlives the call. If you need
  the data afterwards, copy it (`data.to_vec()`).

## Foreign-language types

Each binding maps `[ByRef] bytes` to a natural zero-copy type:

| Binding | Foreign type             | Note                                                          |
|---------|--------------------------|---------------------------------------------------------------|
| Kotlin  | `java.nio.ByteBuffer`    | Must be a *direct* buffer (`ByteBuffer.allocateDirect(...)`). |
| Swift   | `Data`                   | The call runs inside `Data.withUnsafeBytes`.                  |
| Python  | `bytes`                  | The `bytes` object's stable internal pointer is used.         |
| Ruby    | `String(BINARY)`         | Uses `String` with `Encoding::BINARY`.                        |

Passing a non-direct `ByteBuffer` from Kotlin raises an
`IllegalArgumentException` at the call site, matching JNA's
`getDirectBufferPointer` requirement.

## Mutable buffers (`&mut [u8]` / `[ByMutRef] bytes`)

A **synchronous** function can take `&mut [u8]` (or `[ByMutRef] bytes` in
UDL) to borrow a foreign-owned buffer and write to it. The writes land in
the caller's buffer — no copy in or out.

```rust
#[uniffi::export]
pub fn fill(buf: &mut [u8]) {
    // The caller sees these writes once the call returns.
    for (i, b) in buf.iter_mut().enumerate() {
        *b = i as u8;
    }
}
```

```idl
namespace example {
    void fill([ByMutRef] bytes buf);
};
```

Every `&[u8]` rule above still holds. Two more:

- **Synchronous only.** `&mut [u8]` / `[ByMutRef]` in an `async` function
  fails to compile (proc-macro) or generate (UDL). An async call can
  resume on another thread after the caller has moved on and freed the
  buffer.
- **Pass a writable buffer.** Each binding needs a mutable buffer type and
  rejects a read-only one.

Each binding maps a mutable argument to a writable type:

| Binding | Foreign type                 | Note                                                |
|---------|------------------------------|-----------------------------------------------------|
| Kotlin  | direct `java.nio.ByteBuffer` | The same direct buffer as `[ByRef]`; writes land in it. |
| Swift   | `inout Data`                 | The call runs inside `Data.withUnsafeMutableBytes`. |
| Python  | `bytearray`                  | Pass `bytes` and it fails — `bytes` is immutable.   |

Ruby doesn't support `&mut [u8]`.
