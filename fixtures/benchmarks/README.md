This fixture runs a set of benchmark tests, using criterion to test the performance.

Benchmarking UniFFI is tricky and involves a bit of ping-pong between Rust and
the foreign language:

 - `benchmarks.rs` is the top-level Rust executuble where the process starts.
   It parses the CLI arguments and determines which languages we want to run
   the benchmarks for.
 - `benchmarks.rs` executes a script for each foreign language that we want to benchmark.
 - Those scripts call the `run_benchmarks()` function from `lib.rs`
 - `run_benchmarks()` parses the CLI arguments again, this time to determine how to setup
   the `Criterion` object.
 - Testing callback interfaces is relatively straightforward, we simply invoke
   the callback method.
 - Testing regular functions requires some extra care, since these are called
   by the foreign bindings.  To test those, `benchmarks.rs` invokes a callback
   interface method that calls the Rust function and reports the time taken.
