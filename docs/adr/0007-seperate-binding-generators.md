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

This ADR re-evaluates the architecture of how we consume the bindings in hope of improving the developer experience of adding new bindings, testing new and the current bindings and simplifying the ownership of the bindings.
## Decision Drivers

* Maintainability, we would like to improve the maintainability of our binding generators - this means clear ownership of the bindings.
* Testability, it should be easy for developers to test the bindings they care about. Without having to navigate and install unfamiliar libraries and build systems.
* Version compatibility, we need to guarantee that the same version of `uniffi` is used to generate both the scaffolding and the bindings. This is easy to ensure when all the bindings (and the scaffolding) live under the same crate but becomes a concern if they are separated.
* Developer experience, it shouldn't be harder to write and maintain a new binding generator than it currently is.
* Releases, cutting releases for changes in one binding generator shouldn't harm another.

## Considered Options

* **[Option 1] Do nothing**

    This means keeping everything as-is, and deciding that all binding generators (at least for now) should live under the `uniffi_bindgen` crate.

* **[Option 2] Expose a trait in `uniffi_bindgen` that can be implemented by other crates, to separate the implementation of the bindings**

    This means that `uniffi_bindgen` would still handle generic tasks related to binding generation. This includes:
        1. Parsing the UDL, and passing the `ComponentInterface` to the specific binding generator
        1. Calling a generic `BindingGenerator::write_bindings` function that is implemented by the specific binding generator
        1. `uniffi_bindgen` would also expose its `CodeOracle`, `CodeType` and `CodeDeclaration` types to help developers create a standard way to interact with code generation. It shouldn't however, restrict developers to use those concepts if they don't fit a specific binding.
    But developers would need a separate crate that implements the traits `uniffi_bindgen` exposes.

* **[Option 3] Remove `uniffi_bindgen`, replace it with `uniffi_scaffolding` and have each binding generator implementation fully manage the generation of bindings**
    This means external crates would have to depend on `uniffi` directly and implement command-line interfaces. This gives full autonomy to the binding generator developer. Developers won't have access to any traits from `uniffi_bindgen`

## Pros and Cons of the Options

### **[Option 1] Do nothing**

* Good, because it keeps version compatibility easy to guarantee.
* Good, it makes it easier to make changes to multiple bindings at a time (in the case of a breaking change in `uniffi`, etc)
* Bad, because maintainability can grow to be difficult - especially if more bindings are added which the core `uniffi` team is not familiar with
* Bad, Because testability can also grow to be difficult - as more bindings are added (some type of `gecko-js` is expected in the future) the requirement to test all the bindings together in one repository is difficult to maintain.
* Bad, Because releases of all the binding generators are tied to the release of `uniffi_bindgen`

### **[Option 2] Expose a trait in `uniffi_bindgen` that can be implemented by other crates, to separate the implementation of the bindings**

* Good, because maintainability will be clear, and members of the community can opt to maintain their own binding generators
* Good, because testability can improve, where a developer can test a binding generator without building or testing the others
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here)
* Good, because a release in one binding wouldn't have an impact on any other ones
* Bad, because it complicates version compatibility, and we will need more assertions to guarantee it
* Bad, because testability for might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.

Overall this option is preferred because the tradeoffs are clear and manageable and because there is a raising need to create bindings for gecko-js, which can't be tested end-to-end without a complex build system.

### **[Option 3] Remove `uniffi_bindgen`, replace it with `uniffi_scaffolding` and have each binding generator implementation fully manage the generation of bindings**

* Good, because maintainability will be clear, and members of the community can opt to maintain their own binding generators
* Good, because we make it clear that each binding generator is responsible for its own quality
* Bad, because new binding generators can't take advantage of the prior art of the existing binding generators
* Bad, because it will be hard to guarantee the quality of binding generators since they won't depend on any traits from `uniffi_bindgen`
* Bad, because it complicates version compatibility, and we will need more assertions to guarantee it
* Bad, because testability for might increase in complexity. Bindings not in this repository will not have access to the extensive fixtures and examples we have. We might need to expose those if take this option.
* Bad because it increases the number of crates we manage. As we would manage the core crates for the bindings we depend on (i.e kotlin, swift, python, and soon a C++/gecko one). This will make the release process painful, but we can add automation and clear crate ownership to help.



## Decision Outcome

Chosen option:

*  **[Option 2] Expose a trait in `uniffi_bindgen` that can be implemented by other crates, to separate the implementation of the bindings**

### Changes Proposed
#### The exposed trait for binding generators to implement
`unffi_bindgen` would define a public trait, `BindingGenerator` that includes one main function:
```rs
/// Takes in the ComponentInterface and writes the bindings into directory `out_dir`
fn write_bindings<P: AsRef<Path>>(&self, ci: &CompenentInterface, out_dir: P) -> Result<()>;
```

Additionally, we can define optional functions to extend testing functionality
```rs
/// Compiles the generated bindings using `write_bindings`
fn compile_bindings<P: AsRef<Path>>(&self, ci: &ComponentInterface, out_dir: P) -> Result<()>;

/// Runs a script in the language that implements the `BindingGenerator` trait
fn run_script<P: AsRef<Path>>(&self, out_dir: P, script_file: P) -> Result<()>;
```

There are a few specific ways to implement this

## **Implementation Option 1: Have `uniffi_bindgen` shell out to `uniffi_bindgen_{language}`**
Using this approach, we would recommend users to install both the `uniffi_bindgen` **binary** and the `uniffi_bindgen_kotlin` (using kotlin as an example) **binary**. Then, the user would run the following:
1. The user runs `uniffi-bindgen generate -l kotlin -o <OUT_PATH> <UDL_PATH>`
1. `uniffi-bindgen` gets its own version, let's say `0.16` as an example.
1. `uniffi-bingen` shells out to `uniffi-bingen-kotlin -o <OUT_PATH> -v 0.16 <UDL_PATH>`
1. `uniffi-bindgen-kotlin` asserts that it was compiled using the same `0.16` version, otherwise it panics.
1. `uniffi-bindgen-kotlin` then calls a `generate_bindings` function in `uniffi-bindgen` (as a library dependency)
1. `uniffi-bindgen`'s `generate_bindings` takes a generic parameter type that implements a trait `BindingGenerator`.
1. `uniffi-bindgen` parses the `udl`, then calls the generic type's `write_bindings` function (the implementation lives in `uniffi-bingen-kotlin`)


## **Implementation Option 2: Have `uniffi_bindgen` get dynamically linked with exactly one `uniffi_bindgen_{language}`**
This approach uses the specific `uniffi_bindgen_{language}` as plugins. It essentially replaces the shelling out in `Implementation Option 1` with calling an exposed unsafe `C` API in a dynamically linked crate. 

I'm not 100% sure how to let `uniffi_bindgen` choose which crate it gets dynamically linked with on run time, but it's possible.


## **Implementation Option 3: Have each `uniffi_bindgen_{language}` be run directly**
This approach has `uniffi_bindgen` only provide scaffolding and traits for each `uniffi_bindgen_{language}` to implement. `uniffi_bindgen` would also include some generic functions that can be called from `uniffi_bindgen_{language}` by treating `uniffi_bindgen` as a library dependency. The flow would look like this:

1. A user would invoke `uniffi-bindgen-kotlin bindings -o <OUT_PATH> <UDL_PATH>`
1. `uniffi-bindgen-kotlin` would call a generic function `generate_bindings` in `uniffi-bindgen` that takes type parameter `B: BindingGenerator`
1. `uniffi-bindgen` parses the `udl`, then calls the generic type's `write_bindings` function (which lives in `uniffi-bingen-kotlin`)

Version compatibility is a serious consideration with this approach, and we would benefit from exposing a `scaffolding` option to `uniffi-bindgen-kotlin`. This way the same `uniffi-bindgen-kotlin` is used to generate both scaffolding and bindings. We can also add a warning to `uniffi-bindgen`'s scaffolding that we recommend using a specific binary (like `uniffi-bindgen-kotlin`) to generate the scaffolding as well. Note: The scaffolding generating logic would still live in `uniffi-bindgen` we would only expose it to be called. `uniffi-bindgen-kotlin` would not modify the logic at all, this is purely for version compatibility.

<br />
<br />

The old prototype in https://github.com/mozilla/uniffi-rs/pull/997 did Implementation Option 1, **I suggest we do Implementation Option 3 as its easier to reason about.**

<br />
<br />

### Testing
An important consideration for splitting the binding generators is the testing story for crates that live outside of the main `uniffi-rs` repository. We have a high-level decision to make:
Should we expose our fixtures as testing standards? And if yes, how do we do that.
I suggest that we leave this as an open question and open another issue to discuss it specifically.

In the meantime, we can have our own crates consume the fixtures as they do currently since they are all in the same repository.
