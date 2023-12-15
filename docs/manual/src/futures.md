# Async/Future support

UniFFI supports exposing async Rust functions over the FFI. It can convert a Rust `Future`/`async fn` to and from foreign native futures (`async`/`await` in Python/Swift, `suspend fun` in Kotlin etc.)

Check out the [examples](https://github.com/mozilla/uniffi-rs/tree/main/examples/futures) or the more terse and thorough [fixtures](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/futures).

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
