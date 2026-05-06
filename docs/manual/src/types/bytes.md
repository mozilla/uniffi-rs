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

Passing a non-direct `ByteBuffer` from Kotlin raises an
`IllegalArgumentException` at the call site, matching JNA's
`getDirectBufferPointer` requirement.
