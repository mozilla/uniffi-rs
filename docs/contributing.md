# Contributing to UniFFI

Anyone is welcome to help with the UniFFI project. Feel free to get in touch with other community members on Matrix or through issues here on GitHub.

* Matrix: [#uniffi:mozilla.org](https://matrix.to/#/#uniffi:mozilla.org)
* The [issue list](https://github.com/mozilla/uniffi-rs/issues)

Participation in this project is governed by the
[Mozilla Community Participation Guidelines](https://www.mozilla.org/en-US/about/governance/policies/participation/).

## Building the project

You will need a working [Rust](https://www.rust-lang.org/) development environment in order to build this project.
We recommend using [`rustup`](https://rustup.rs/) to install the latest Rust toolchain and stay up-to-date with releases.

In the root of your repository checkout, run:

```
cargo build
```

If you have `rustup` installed then this should work without errors; please [file an issue](https://github.com/mozilla/uniffi-rs/issues)
if it doesn't work for you.

## Running the tests

In the root of your repository checkout, run:

```
cargo test
```

This should run, but is very likely to report test failures.

Since this project generates API bindings for multiple languages, running the full test suite requires
a working host environment for multiple languages.

We publish a Docker image with the necessary dependencies pre-installed, and you can run `cargo test`
from inside this Docker image using a helper script like so:

```
./docker/cargo-docker.sh test
```

This is convenient, but slower than running `cargo` locally. For rapid testing and iteration we
recommend installing the language dependencies on your local machine. In order to locally run the full
test suite you will need:

* Kotlin:
  * `kotlinc`, the [Kotlin command-line compiler](https://kotlinlang.org/docs/command-line.html).
  * `ktlint`, the [Kotlin linter used to format the generated bindings](https://ktlint.github.io/).
  * The [Java Native Access](https://github.com/java-native-access/jna#download) JAR downloaded and its path
    added to your `$CLASSPATH` environment variable.
* Swift:
  * `swift` and `swiftc`, the [Swift command-line tools](https://swift.org/download/).
  * The Swift `Foundation` package.
* Python:
  * A `python3` interpreter.
* Ruby:
  * A `ruby` interpreter.
  * The [`FFI`](https://github.com/ffi/ffi) Ruby gem, installable via `gem install ffi`.

We also support an environment variable `UNIFFI_TESTS_DISABLE_EXTENSIONS`;
It is a set of file extensions, without a leading period and separated by commas.
Eg, `UNIFFI_TESTS_DISABLE_EXTENSIONS=swift,rb cargo test` will skip test filenames ending in
`.swift` or `.rb`

## Navigating the code

If you're new to UniFFI, we recommend starting with the example projects in the [`./examples` directory](../examples/).
These will give you an idea of how the tool is used in practice. In particular, take a look in the `./tests/bindings/`
subdirectory of each example to see how its generated API can be used from different target languages.

Other directories of interest include:

- **[`./uniffi_bindgen`](../uniffi_bindgen):** This is the source for the `uniffi-bindgen` executable and is where
  most of the logic for the UniFFI tool lives. Its contents include:
    - **[`./uniffi_bindgen/src/interface/`](../uniffi_bindgen/src/interface):** The logic for parsing `.udl` files
      into an in-memory representation called `ComponentInterface`, from which we can generate code for different languages.
    - **[`./uniffi_bindgen/src/scaffolding`](../uniffi_bindgen/src/scaffolding):** This module turns a `ComponentInterface`
      into *Rust scaffolding*, the code that wraps the user-provided Rust code and exposes it via a C-compatible FFI layer.
    - **[`./uniffi_bindgen/src/bindings/`](../uniffi_bindgen/src/bindings):** This module turns a `ComponentInterface` into
      *foreign-language bindings*, the code that can load the FFI layer exposed by the scaffolding and expose it as a
      higher-level API in a target language. There is a sub-module for each supported language.
- **[`./uniffi`](../uniffi):** This is a run-time support crate that is used by the generated Rust scaffolding. It
  controls how values of various types are passed back-and-forth over the FFI layer, by means of the `FfiConverter` trait.
- **[`./uniffi_build`](../uniffi_build):** This is a small hook to run `uniffi-bindgen` from the `build.rs` script
  of a UniFFI component, in order to automatically generate the Rust scaffolding as part of its build process.
- **[`./uniffi_macros`](../uniffi_macros):** This contains some helper macros that UniFFI components can use to
  simplify loading the generated scaffolding, and executing foreign-language tests.
- **[`./fixtures`](../fixtures):** These are various test fixtures which we use to ensure good test coverage and
  guard against regressions.


## Finding issues to work on

Below are a few different queries you can use to find appropriate issues to work on.
Feel free to reach out if you need any additional clarification before picking up an issue.

- **[`good first issue`](https://github.com/mozilla/uniffi-rs/issues?q=is%3Aopen+is%3Aissue+label%3Agood%20first%20issue)**
    - These are relatively small self-contained issues, suitable for first-time contributors to help them get familiar with
      working on the project and working in Rust.
- **[`get involved`](https://github.com/mozilla/application-services/labels/good-second-issue)**
    - These are larger issues that may touch more parts of the codebase and assume familiarity with working in Rust,
      but with some mentoring notes from experienced developers.

Of course, you are also welcome to "scratch your own itch". If you have a significant change that you'd like to propose
to UniFFI, please start by [opening a new issue with the **`discuss`** label](https://github.com/mozilla/uniffi-rs/issues/new?labels=discuss)
so we can help you figure out how to proceed.


## Sending Pull Request

Changes should be submitted as [pull requests](https://help.github.com/articles/about-pull-requests/) (PRs).

Before submitting a PR:
- Your patch should include new tests that cover your changes, or be accompanied by explanation for why it doesn't need any. It is your and your reviewer's responsibility to ensure your patch includes adequate tests.
- Your code should pass all the automated tests before you submit your PR for review.
  - See [Running the tests](#running-the-tests) above.
  - "Work in progress" pull requests are welcome, but should be clearly labeled as such and should not be merged until all tests pass and the code has been reviewed.
    - You can label pull requests as "Work in progress" by using the Github PR UI to indicate this PR is a draft ([learn more about draft PRs](https://docs.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests#draft-pull-requests)).
- Run `cargo fmt` to ensure your Rust code is correctly formatted. You should run this command after running tests and before pushing changes so that any fixes for failed tests are included.
- Your patch should include a changelog entry in the "unreleased" section of [CHANGES.md](../CHANGES.md), particularly
  if it would be a breaking change for consumers of the tool.
- If your patch adds new dependencies, they must follow our [dependency management guidelines](./dependency-management.md).
  Please include a summary of the due dilligence applied in selecting new dependencies.

When submitting a PR:
- You agree to license your code under the project's open source license ([MPL 2.0](/LICENSE)).
- Base your branch off the current `main` branch.
- Add both your code and new tests if relevant.
- Please do not include merge commits in pull requests; include only commits with the new relevant code.
- We encourage you to [GPG sign your commits](https://help.github.com/articles/managing-commit-signature-verification).

### Code Review

This project is production Mozilla code and subject to our [code-review requirements](https://firefox-source-docs.mozilla.org/contributing/Code_Review_FAQ.html).
Every change must be reviewed and approved by a member with write access to the main `mozilla/uniffi-rs` repository.

### Merging code

Pull requests can be merged if all tests are passing and it got at least one approving review from a member of the `@mozilla/uniffi-devs` team.
We make use of GitHub's functionality to [automatically merge a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request).
Enabling that is the task of the Pull Request author.

For Pull Requests from outside contributors the reviewer should merge the Pull Request upon a successful review or should enable the automatic merge if tests have not yet finished.