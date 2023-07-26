# Async/Future support

UniFFI supports exposing async Rust functions over the FFI. It can convert a Rust `Future`/`async fn` to and from foreign native futures (`async`/`await` in Python/Swift, `suspend fun` in Kotlin etc.)

Check out the [examples](https://github.com/mozilla/uniffi-rs/tree/main/examples/futures) or the more terse and thorough [fixtures](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/futures).

Note that currently async functions are only supported by proc-macros, if you require UDL support please file a bug.

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

This code uses `asyncio` to drive the future to completion, while our exposed function is used with `await`.

There are [some great API docs](https://docs.rs/uniffi_core/latest/uniffi_core/ffi/rustfuture/index.html) on the implementation that are well worth a read.

## Foreign executors

UniFFI futures are normally usually driven by a foreign executor - think event-loop, or async runtime.
In the example above it's `asyncio`.
There's no requirement for a Rust executor like tokio, although you can manually start one and use it alongside the foreign executor.
Rust code can also use foreign executors to schedule tasks, for example to schedule work in a background thread.

### Constructing ForeignExecutor instances in foreign languages

- Kotlin uses a `CoroutineScope` (for example: `CoroutineScope(Dispatchers.IO)`)
- Python uses an `EventLoop` (for example: `asyncio.get_running_loop()`)
- Swift uses the UniFFI-defined class `UniFfiForeignExecutor` (for example: `UniFfiForeignExecutor(priority: TaskPriority.background)`)

### Using ForeignExecutor instances in Rust

Use the `uniffi::run!()` and `uniffi::schedule!()` macros to schedule closures to be run using a
`ForeignExecutor`.

  - `run()` schedules a closure to be run, returing a Future.
  - `schedule()` does the same, without returning a Future -- useful for "fire-and-forget" style tasks.

```rust
/// UniFFI-exposed interface that loads data from a database
#[derive(uniffi::Object)]
struct DataStore {
    background_executor: uniffi::ForeignExecutor,
    db: MyDatabase,
}

#[uniffi::export]
impl DataStore {
    /// Construct a DataStore.
    ///
    /// background_executor is a ForeignExecutor that runs tasks in a background thread.
    pub fn new(background_executor: uniffi::ForeignExecutor, db_path: String) -> Arc<Self> {
        Arc::new(Self {
            background_executor,
            db: MyDatabase::new(db_path),
        })
    }

    /// Load the entry using the background executor 
    pub async fn load_entry(self: Arc<Self>) -> DataEntry {
        // use run! to schedule the task
        // Use a move closure to move the Arc<Self> to the background thread.
        uniffi::run!(self.background_executor, move || self.db.load_entry())
    }
}

```

### Further reading

For more info see the [foreignexecutor module documentation](https://docs.rs/uniffi_core/latest/uniffi_core/ffi/foreignexecutor/index.html) and the [futures example crate](https://github.com/mozilla/uniffi-rs/tree/main/examples/futures).

## How it works

As [described in the documentation](https://docs.rs/uniffi_core/latest/uniffi_core/ffi/rustfuture/index.html),
UniFFI generates code which uses callbacks from Rust futures back into that foreign "executor" to drive them to completion.
Fortunately, each of the bindings and Rust have similar models, so the discussion below is Python, but it's almost exactly the same in Kotlin and Swift.

In the above example, the generated `say_after` function looks something like:

```python

# A helper to work with asyncio.
def _rust_say_after_executor(eventloop_handle, rust_task_handle):
    event_loop = UniFFIMagic_GetExecutor(eventloop_handle)

    def callback(task_handle):
        # The event-loop has called us - call back into Rust.
        _uniffi_say_after_executor_callback(task_handle)

    # Now have the asyncio eventloop - ask it to schedule a call to help drive the Rust future.
    eventloop.call_soon_threadsafe(callback, rust_task_handle)

# A helper for say_after which creates a future and passes it Rust
def _rust_call_say_after(callback_fn):
    # Handle to our executor.
    eventloop = asyncio.get_running_loop()
    eventloop_handle = UniFFIMagic_SetExecutor(eventloop)

    # Use asyncio to create a new Python future.
    future = eventloop.create_future()
    future_handle = UniFFIMagic_SetFuture(future)

    # This is a "normal" UniFFI call across the FFI to Rust scaffoloding, but
    # because it is an async function it has a special signature which
    # requires the handles and the callback.
    _uniffi_call_say_after(executor_handle, callback_fun, future_handle)

    # and return the future to the caller.
    return future

def say_after_callback(future_handle, result)
    future = UniFFIMagic_GetFuture(future_handle)
    if future.cancelled():
        return
    future.set_result(result))

def say_after(...):
    return await _rust_call_say_after(say_after_callback)

```

And the code generated for Rust is something like:

```rust
struct SayAfterHelper {
    rust_future: Future<>,
    uniffi_executor_handle: ::uniffi::ForeignExecutorHandle,
    uniffi_callback: ::uniffi::FfiConverter::FutureCallback,
    uniffi_future_handle: ...,
}

impl SayAfterHelper {
    fn wake(&self) {
        match self.rust_future.poll() {
            Some(Poll::Pending) => {
                // ... snip executor stuff
                self.rust_future.wake()
            },
            Some(Poll::Ready(v)) => {
                // ready - tell the foreign executor
                UniFFI_Magic_Invoke_Foreign_Callback(self.uniffi_callback, self.uniffi_future_handle)
            },
            None => todo!("error handling"),
        }
    }
}

pub extern "C" fn _uniffi_call_say_after(
    uniffi_executor_handle: ::uniffi::ForeignExecutorHandle,
    uniffi_callback: ::uniffi::FfiConverter::FutureCallback,
    uniffi_future_handle: ...,
) {
    // Call the async function to get the Rust future.
    let rust_future = say_after(...)
    let helper = SayAfterHelper {
        rust_future,
        uniffi_executor_handle,
        uniffi_callback,
        uniffi_future_handle,
    );
    helper.wake();
    Ok(())
}
```
