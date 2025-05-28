# Async/Future support

UniFFI supports exposing async Rust functions over the FFI. It can convert a Rust `Future`/`async fn` to and from foreign native futures (`async`/`await` in Python/Swift, `suspend fun` in Kotlin etc.)

Check out the [examples](https://github.com/mozilla/uniffi-rs/tree/main/examples/futures) or the more terse and thorough [fixtures](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/futures).

We've also [documentation on the internals of how this works](internals/async-overview.md).

## Example

This is a short "async sleep()" example:
```Rust
use std::time::Duration;
use async_std::future::{timeout, pending};

/// Async function that says something after a certain time.
#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    let never = pending::<()>();
    timeout(Duration::from_millis(ms), never).await.unwrap_err();
    format!("Hello, {who}!")
}
```

This can be called by the following Python code:
```python
import asyncio
from uniffi_example_futures import *

async def main():
    print(await say_after(20, 'Alice'))

if __name__ == '__main__':
    asyncio.run(main())
```

Async functions can also be defined in UDL:
```idl
namespace example {
    [Async]
    string say_after(u64 ms, string who);
}
```

This code uses `asyncio` to drive the future to completion, while our exposed function is used with `await`.

In Rust `Future` terminology this means the foreign bindings supply the "executor" - think event-loop, or async runtime. In this example it's `asyncio`. There's no requirement for a Rust event loop.

There are [some great API docs](https://docs.rs/uniffi_core/latest/uniffi_core/ffi/rustfuture/index.html) on the implementation that are well worth a read.

## Exporting async trait methods

UniFFI is compatible with the [async-trait](https://crates.io/crates/async-trait) crate and this can
be used to export trait interfaces over the FFI.

When using UDL, wrap your trait with the `#[async_trait]` attribute.  In the UDL, annotate all async
methods with `[Async]`:

```idl
[Trait]
interface SayAfterTrait {
    [Async]
    string say_after(u16 ms, string who);
};
```

When using proc-macros, make sure to put `#[uniffi::export]` outside the `#[async_trait]` attribute:

```rust
#[uniffi::export]
#[async_trait::async_trait]
pub trait SayAfterTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}
```

## Combining Rust and foreign async code

Traits with callback interface support that export async methods can be combined with async Rust code.
See the [async-api-client example](https://github.com/mozilla/uniffi-rs/tree/main/examples/async-api-client) for an example of this.

### Python: `uniffi_set_event_loop()`

Python bindings export a function named `uniffi_set_event_loop()` which handles a corner case when
integrating async Rust and Python code. `uniffi_set_event_loop()` is needed when Python async
functions run outside of the eventloop, for example:

* Rust code is executing outside of the eventloop.  Some examples:
    * Rust code spawned its own thread
    * Python scheduled the Rust code using `EventLoop.run_in_executor`
* The Rust code calls a Python async callback method, using something like `pollster` to block
  on the async call.

In this case, we need an event loop to run the Python async function, but there's no eventloop set for the thread.
Use `uniffi_set_event_loop()` to handle this case.
It should be called before the Rust code makes the async call and passed an eventloop to use.

Note that `uniffi_set_event_loop` cannot be glob-imported because it's not part of the library's `__all__`.

## Cancelling async code.

We don't directly support cancellation in UniFFI even when the underlying platforms do.
You should build your cancellation in a separate, library specific channel; for example, exposing a `cancel()` method that sets a flag that the library checks periodically.

Cancellation can then be exposed in the API and be mapped to one of the error variants, or None/empty-vec/whatever makes sense.
There's no builtin way to cancel a future, nor to cause/raise a platform native async cancellation error (eg, a swift `CancellationError`).

See also [this github PR](https://github.com/mozilla/uniffi-rs/pull/1768).
