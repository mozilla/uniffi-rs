# Async/Future support

UniFFI supports exposing async rust functions over the FFI. It can convert a Rust `Future`/`async fn` to and from foreign native futures (`async`/`await` in Python/Swift, `suspend fun` in Kotlin etc.)

Check out the [examples](https://github.com/mozilla/uniffi-rs/tree/main/examples/futures) or the more terse and thorough [fixtures](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/futures).

Note that currently async functions are only supported by proc-macros, but UDL would be fairly simple enhancement.

## Example

This is the shortest "async sleep()" I could come up with!
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

In Rust `Future` terminology, this means the foreign bindings supply the "executor" - think event-loop, or async runtime. In this example it's `asyncio`. There's no requirement for a Rust event loop.

See the [foreign-executor fixture](https://github.com/mozilla/uniffi-rs/tree/main/fixtures/foreign-executor) for more implementation details.

## How it works

UniFFI generates code which uses callbacks from Rust futures back into that foreign "executor" to drive them to completion.
Fortunately, each of the bindings and Rust have consistent models, so the discussion below is Python, but it's almost exactly the same in Kotlin and Swift.

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
    uniffi_rust_future.wake();
    Ok(())
};

```