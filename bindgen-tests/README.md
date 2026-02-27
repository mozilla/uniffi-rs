# bindgen-tests

These are a new test suite for UniFFI that we're currently working on switching to.
They replace the older tests in fixtures/examples.

The main difference is that these tests don't try to represent any kind of real-world scenario.
Instead, they try to directly exercise the bindings code with artificial cases.
This has a few advantages:

* You can test new bindgen code step-by-step as you build it.
  Each test targets a particular feature, and you can enable them one-by-one using crate features.
  For example, start with `simple_fns`, then move to `primitive_types`, `records`, `enums`, etc.
* It's more obvious why things fail. Since each test is very specific, you can more easily know what failed.
  If the `records` test fails, you can be pretty sure you have an issue with your record-related code.
  When the `examples/todolist` test fails, it's not so clear.
* There's less crates involved.
  If we wanted, we could publish the 2 test crates involved without too much overhead.
  Publishing all of the examples/fixtures is a lot.
* Simplified test harness code.
  This kind-of follows from the last points.
  Since we only need to build a single library, we don't need so much abstraction.
* Language-specific tests get their own directory.
  For example, the Swift-specific tests all live in the `swift/` subdirectory.
  If we decide to split `uniffi-bindgen-swift` into it's own crate,
  this will make it easier for the tests to move there.

## Running the tests

* Go to one of the language directories (for example `bindgen-tests/swift`)
* `./cargo test`
* ...or something like `./cargo test test_records` to test a specific feature

## Adding tests for a new feature

* Create a library types/functions to the test feature
  * Create a new module in `bindgen-tests/lib/src/`
  * Add a feature flag for it
  * Add a `pub mod` statement in `bindgen-tests/lib/src/lib.rs`, conditional on that feature flag.
* Add tests for each language
    * Swift
        * Add a Swift test case file in `bindgen-tests/swift/tests`
        * Add a test function for the new test case in `bindgen-tests/swift/src/lib.rs`
    * Kotlin: TODO
    * Python: TODO
    * Ruby: TODO

## Migration plan

The tests are only currently supported for Swift, the next step is making them work for the other
bindings languages.

After that, we'll want to keep the existing fixtures for a while, since these tests may not be testing everything.
New tests should be added here if possible.
If we see a failure in one of the existing fixtures, we should update these tests to also catch it.
We should review the existing fixtures, think if these tests are missing anything from them, and update them if so.

At some point, we'll be confident that the older fixtures aren't needed anymore and we can start to get rid of them.
We'll still want to keep the examples around, since they serve as a form of documentation.
We may also decide to keep running tests for them, since they could serve as a form of end-to-end testing.
