# Separate the binding generators into their own crates per target

* Status: proposed
* Deciders: TBD
* Date: 2022-01-16

Technical Story: [Issue 299](https://github.com/mozilla/uniffi-rs/issues/299)

outdated early prototype: [PR 997](https://github.com/mozilla/uniffi-rs/pull/997)

## Context and Problem Statement
All the binding generators currently live in the [`uniffi_bindgen`](../../uniffi_bindgen/src/bindings) crate. This creates the following difficulties:

- All the bindings live in the `uniffi` repository, so the `uniffi` team has to maintain them (or at the very least review changes to them).
- Any changes to a specific binding generator require a new `uniffi_bindgen` release for it to be released. Even if it doesn't impact any of the other bindings.
- Some bindings require complex build systems to test, and including those build systems in `uniffi` adds its complexity. For example, any type of `gecko-js` bindings would require the mozilla-central build system to build and test.
- We currently run all the tests for the bindings in our CI and through `cargo test`. This means that if one binding target gets outdated and fails, or if a developer doesn't have the needed libraries installed for one of the targets, the tests would fail.
- It's currently difficult to support third-party developers writing bindings for languages the core team does not care about.

This ADR re-evaluates the architecture of how we consume the bindings in hope of improving the developer experience of adding new bindings, testing new and the current bindings and simplifying the ownership of the bindings.
## Decision Drivers

* Maintainability, we would like to improve the maintainability of our binding generators - this means clear ownership of the bindings.
* Testability, it should be easy for developers to test the bindings they care about. Without having to navigate and install unfamiliar libraries and build systems.
* Version compatibility, this refers to the same version of `uniffi` being used to generate both the scaffolding and the bindings. This includes:
    - How easy is it to accidentally have a version mismatch
    - How much damage is caused by the the version mismatch
    - The difficulty of debugging a version mismatch
* Developer experience, it should be easier to write and maintain a new binding generator than it currently is.
* Releases, cutting releases for changes in one binding generator shouldn't harm another.

## Considered Options

* **[Option 1] Do nothing**

    This means keeping everything as-is, and deciding that all binding generators (at least for now) should live under the `uniffi_bindgen` crate.

* **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

    Users would have to install a binary of the crates of the binding language they would like to user **in addition** to installing `uniffi_bindgen`. For example, a user would need to:

    1. run `cargo install uniffi_bindgen_kotlin`
    1. run `cargo install uniffi_bindgen`
    1. Then a user will finally be able to run `uniffi_bindgen generate -l kotlin <ARGS>`. User must install `uniffi_bindgen_kotlin` otherwise `uniffi_bindgen` would complain

    This means that `uniffi_bindgen` would still handle generic tasks related to binding generation. This includes:

    1. Parsing the UDL, and passing the `ComponentInterface` to the specific binding generator
    1. Parsing the `uniffi.toml` and passing the language specific configuration down to the binding generator
    1. Calling a generic `BindingGenerator::write_bindings` function that is implemented by the specific binding generator
    1. `uniffi_bindgen` would also expose its `CodeOracle`, `CodeType` and `CodeDeclaration` types to help developers create a standard way to interact with code generation. It shouldn't however, restrict developers to use those concepts if they don't fit a specific binding.
    But developers would need a separate crate that implements the traits `uniffi_bindgen` exposes.

* **[Option 3] Publish a separate crate per binding generator and have them get consumed directly**

    Users would only need to install `uniffi_bindgen_{lang}` and run it like `uniffi_bindgen_kotlin <ARGS>`, and each `uniffi_bindgen_{lang}` could opt to support generating scaffolding as well.
## Pros and Cons of the Options

### **[Option 1] Do nothing**

* Good, because it makes it hard to accidentally use different versions of `uniffi` for scaffolding and bindings
* Good, it makes it easier to make changes to multiple bindings at a time (in the case of a breaking change in `uniffi`, etc)
* Bad, because maintainability can grow to be difficult - especially if more bindings are added which the core `uniffi` team is not familiar with
* Bad, because testability can also grow to be difficult - as more bindings are added (some type of `gecko-js` is expected in the future) the requirement to test all the bindings together in one repository is difficult to maintain.
* Bad, Because releases of all the binding generators are tied to the release of `uniffi_bindgen`

### **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

* Good, because maintainability will be clear, and members of the community can opt to maintain their own binding generators
* Good, because testability can improve, where a developer can test a binding generator without building or testing the others
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here)
* Good, because a release in one binding wouldn't have an impact on any other ones unless it changes internal `uniffi` behavior.
* Good, because it makes it easier to catch a version mismatch between scaffolding and bindings - since the version is passed between the binaries.
* Bad, because it's easier to accidentally have a version mismatch.
* Bad, because testability might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.

Overall this option is preferred because:

- The tradeoffs are clear and manageable and because there is a raising need to create bindings for gecko-js, which can't be tested end-to-end without a complex build system.
- The benefit from losing that extra hop out of uniffi_bindgen isn't worth having our consumers try to enforce compatibility across different `uniffi_bindgen_{lang}` (especially like Glean and App Services where we have multiple bindings at a time)
- `uniffi_bindgen` staying as the source of truth (with it passing it's version to the other crates) has a good benefit of reducing metal load to keep track of which binary to call


### **[Option 3] Publish a separate crate per binding generator and have them get consumed directly**

* Good, because maintainability will be clear, and members of the community can opt to maintain their own binding generators
* Good, because testability can improve, where a developer can test a binding generator without building or testing the others
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here)
* Good, because a release in one binding wouldn't have an impact on any other ones unless it changes internal `uniffi` behavior.
* Bad, because it's easier to accidentally have a version mismatch. And we have even less flexibility than `Option 2` to mitigate it since `uniffi_bindgen` won't shell out to the crate.
* Bad, because it's not clear how the users should generate the scaffolding, and if they use `uniffi_bindgen` there is a risk of version incompatibility.
* Bad, because testability might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.


## Decision Outcome

Chosen option:

* **[Option 2] Publish a separate crate per binding generator and have users use the  `uniffi_bindgen` Command line interface to shell out to the specific binding generators**

### Changes Proposed
#### The exposed trait for binding generators to implement
`unffi_bindgen` would define a public trait, `BindingGenerator` that includes one main function:
```rs
/// Takes in the ComponentInterface and Config and writes the bindings into directory `out_dir`
fn write_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, config: Config, out_dir: P) -> Result<()>;
```

Additionally, we can define optional functions to extend testing functionality
```rs
/// Compiles the generated bindings using `write_bindings`
fn compile_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, out_dir: P) -> Result<()>;

/// Runs a script in the language that implements the `BindingGenerator` trait
fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> Result<()>;
```

Using this approach, we would recommend users to install both the `uniffi_bindgen` **binary** and the `uniffi_bindgen_kotlin` (using kotlin as an example) **binary**. Then, the user would run the following:
1. The user runs `uniffi-bindgen generate -l kotlin -o <OUT_PATH> <UDL_PATH>`
1. `uniffi-bindgen` gets its own version, let's say `0.16` as an example.
1. `uniffi-bingen` shells out to `uniffi-bingen-kotlin -o <OUT_PATH> -v 0.16 <UDL_PATH>`
1. `uniffi-bindgen-kotlin` asserts that it was compiled using the same `0.16` version, otherwise it panics.
1. `uniffi-bindgen-kotlin` then calls a `generate_bindings` function in `uniffi-bindgen` (as a library dependency)
1. `uniffi-bindgen`'s `generate_bindings` takes a generic parameter type that implements a trait `BindingGenerator`.
1. `uniffi-bindgen` parses the `udl`, then calls the generic type's `write_bindings` function (the implementation lives in `uniffi-bingen-kotlin`)

<br />
<br />

### Testing
An important consideration for splitting the binding generators is the testing story for crates that live outside of the main `uniffi-rs` repository. We have a high-level decision to make:
Should we expose our fixtures as testing standards? And if yes, how do we do that.
At the time of this ADR, [we created a ticket on Uniffi to discuss this as its own issue](https://github.com/mozilla/uniffi-rs/issues/1151)
In the meantime, we can have our own crates consume the fixtures as they do currently since they are all in the same repository.
