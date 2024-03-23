This crate is a toy build an async API client, with some parts implemented in Rust and some parts
implemented in the foreign language.  Each side makes async calls across the FFI.

The motivation is to show how to build an async-based Rust library, using a foreign async executor to drive the futures.
Note that the Rust code does not start any threads of its own, nor does it use startup an async runtime like tokio.
Instead, it awaits async calls to the foreign code and the foreign executor manages the threads.

There are two basic ways the Rust code in this crate awaits the foreign code:

## API calls

API calls are the simple case.
Rust awaits an HTTP call to the foreign side, then uses `serde` to parse the JSON into a structured response.
As long as the Rust code is "non-blocking" this system should work fine.
Note: there is not a strict definition for "non-blocking", but typically it means not performing IO and not executing a long-running CPU operation.

## Blocking tasks

The more difficult case is a blocking Rust call.
The example from this crate is reading the API credentials from disk.
The `tasks.rs` module and the foreign implementations of the `TaskRunner` interface are an experiment to show how this can be accomplished using async callback methods.

The code works, but is a bit clunky.
For example requiring that the task closure is `'static` creates some extra work for the `load_credentials` function.
It also requires an extra `Mutex` and `Arc`.

The UniFFI team is looking for ways to simplify this process by handling it natively in UniFFI, see https://github.com/mozilla/uniffi-rs/pull/1837.
If you are writing Rust code that needs to make async blocking calls, please tell us about your use case which will help us develop the feature.
