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

## Combining Rust and foreign async code

Traits with callback interface support that export async methods can be combined with async Rust code.
See the [async-api-client example](https://github.com/mozilla/uniffi-rs/tree/main/examples/async-api-client) for an example of this.

### Python: uniffi_set_event_loop()

Python bindings export a function named `uniffi_set_event_loop()` which handles a corner case when
integrating async Rust and Python code. `uniffi_set_event_loop()` is needed when Python async
functions run outside of the eventloop, for example:

    - Rust code is executing outside of the eventloop.  Some examples:
        - Rust code spawned its own thread
        - Python scheduled the Rust code using `EventLoop.run_in_executor`
    - The Rust code calls a Python async callback method, using something like `pollster` to block
      on the async call.

In this case, we need an event loop to run the Python async function, but there's no eventloop set for the thread.
Use `uniffi_set_event_loop()` to handle this case.
It should be called before the Rust code makes the async call and passed an eventloop to use.

## Blocking tasks

Rust executors are designed around an assumption that the `Future::poll` function will return quickly.
This assumption, combined with cooperative scheduling, allows for a large number of futures to be handled by a small number of threads.
Foreign executors make similar assumptions and sometimes more extreme ones.
For example, the Python eventloop is single threaded -- if any task spends a long time between `await` points, then it will block all other tasks from progressing.

This raises the question of how async code can interact with blocking code that performs blocking IO, long-running computations without `await` breaks, etc.
UniFFI defines the `BlockingTaskQueue` type, which is a foreign object that schedules work on a thread where it's okay to block.

On Rust, `BlockingTaskQueue` is a UniFFI type that can safely run blocking code.
It's `execute` method works like tokio's [block_in_place](https://docs.rs/tokio/latest/tokio/task/fn.block_in_place.html) function.
It inputs a closure and runs it in the `BlockingTaskQueue`.
This closure can reference the outside scope (i.e. it does not need to be `'static`).
For example:

```rust
#[derive(uniffi::Object)]
struct DataStore {
  // Used to run blocking tasks
  queue: uniffi::BlockingTaskQueue,
  // Low-level DB object with blocking methods
  db: Mutex<Database>,
}

#[uniffi::export]
impl DataStore {
  #[uniffi::constructor]
  fn new(queue: uniffi::BlockingTaskQueue) -> Self {
      Self {
          queue,
          db: Mutex::new(Database::new())
      }
  }

  async fn fetch_all_items(&self) -> Vec<DbItem> {
     self.queue.execute(|| self.db.lock().fetch_all_items()).await
  }
}
```

On the foreign side `BlockingTaskQueue` corresponds to a language-dependent class.

### Kotlin
Kotlin uses `CoroutineContext` for its `BlockingTaskQueue`.
Any `CoroutineContext` will work, but `Dispatchers.IO` is usually a good choice.
A DataStore from the example above can be created with `DataStore(Dispatchers.IO)`.

### Swift
Swift uses `DispatchQueue` for its `BlockingTaskQueue`.
The user-initiated global queue is normally a good choice.
A DataStore from the example above can be created with `DataStore(queue: DispatchQueue.global(qos: .userInitiated)`.
The `DispatchQueue` should be concurrent.

### Python

Python uses a `futures.Executor` for its `BlockingTaskQueue`.
`ThreadPoolExecutor` is typically a good choice.
A DataStore from the example above can be created with `DataStore(ThreadPoolExecutor())`.
