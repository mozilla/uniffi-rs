This fixture runs a set of benchmark tests, using criterion to test the performance.

- `cargo uniffi-bench` to run all benchmarks.
- `cargo uniffi-bench [filter-string]` to run a subset of the benchmarks
- `cargo uniffi-bench --help` for more details on the CLI

## Creating benchmark tables

`cargo uniffi-bench` can create tables to compare multiple branches.
Use the `--save`/`-s` and `--compare`/`-c` flags to do this.
For example, to generate a table that compares 3 branches:

- checkout branch1
- `cargo uniffi-bench [args] --save branch1`
- checkout branch2
- `cargo uniffi-bench [args] --save branch2`
- checkout branch3
- `cargo uniffi-bench [args] --save branch3 --compare branch1,branch2,branch3`

## Profiling a benchmark using the Firefox profiler

You can use the `perf` command alongside the Firefox profiler to debug benchmark performance:

Run the bechmarks using `perf record` and with the `--profile-time` flag, for example:

```
perf record -g -F 999 cargo uniffi-bench --profile-time 5 kotlin-rust-call-only
```

Generate a profile from the raw perf data and copy it to some temporary location:

```
perf script -F +pid > ~/Downloads/test.perf
```

Go to https://profiler.firefox.com/ and open the file you just generated.
Focus on the last 5 seconds or so of data and ignore the begining data
which is tracks compiling the benchmarks crate.

## Behind the scenes

Benchmarking UniFFI is tricky and involves a bit of ping-pong between Rust and
the foreign language:

 - `benchmarks.rs` is the top-level Rust executable where the process starts.
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
