# Enable implementing bindings in separate crates.

* Status: Accepted
* Deciders: teshaq, bdk, mhammond
* Date: 2022-04-7

Technical Story: [Issue 299](https://github.com/mozilla/uniffi-rs/issues/299)

Implementation: [PR 1201](https://github.com/mozilla/uniffi-rs/pull/1201)

Testing Implementation: [PR 1206](https://github.com/mozilla/uniffi-rs/pull/1206)

## Context and Problem Statement
All the binding generators currently live in the [`uniffi_bindgen`](../../uniffi_bindgen/src/bindings) crate. This creates the following difficulties:

- All the bindings live in the `uniffi` repository, so the `uniffi` team has to maintain them (or at the very least review changes to them). This makes it difficult to support third-party developers writing bindings for languages the core team does not wish to maintain.
- Any change to a specific binding generator requires a new `uniffi_bindgen` release for it to be accessible by consumers. Even if it doesn't impact any of the other bindings.
- Some bindings require complex build systems to test. Including those build systems in `uniffi` would require developers to install those build systems, and CI to do the same. For example, any type of `gecko-js` bindings would require the mozilla-central build system to build and test.
- We currently run all the tests for the bindings in our CI and through `cargo test`. This means that if one binding target gets outdated and fails, or if a developer doesn't have the needed libraries installed for one of the targets, the tests would fail.

Before [PR 1201](https://github.com/mozilla/uniffi-rs/pull/1201), it was also impossible to write new bindings that did not live in the [`uniffi_bindgen`](../../uniffi_bindgen/src/bindings) crate.

This ADR proposes enabling third-party crates to implement binding generators, and describes the necessary uniffi changes to enable this.
## Decision Drivers

* Support Firefox Desktop JavaScript binding generation
* Testability, it should be easy for developers to test the bindings they care about. Without having to navigate and install unfamiliar libraries and build systems.
* Developer experience, it should be easier to write and maintain a new binding generator than it currently is.
* Releases, cutting releases for changes in one binding generator shouldn't harm another.

**NOTE**: Version compatibility is handled in a [separate ADR](https://github.com/mozilla/uniffi-rs/pull/1203)

## Considered Options

* **[Option 1] Do nothing**

    This means keeping everything as-is, and deciding that all binding generators (at least for now) should live under the `uniffi_bindgen` crate.

* **[Option 2] Create a public API for external crates to implement their own bindings.**

    Developers would have traits exposed they can leverage to implement binding generators that do not live in `uniffi_bindgen`. `uniffi_bindgen` would still handle generic tasks related to binding generation.
## Pros and Cons of the Options

### **[Option 1] Do nothing**

* Good, because it makes it harder for users to accidentally use different versions of `uniffi` for scaffolding and bindings since they are all implemented together in the same crate.
* Good, it makes it easier to make changes to multiple bindings at a time (in the case of a breaking change in `uniffi`, etc).
* Bad, because maintainability can grow to be difficult - especially if more bindings are added which the core `uniffi` team is not familiar with.
* Bad, because testability can also grow to be difficult - as more bindings are added the requirement to test all the bindings together in one repository is difficult to maintain.
* Bad, because releases of all the binding generators are tied to the release of `uniffi_bindgen`.

### **[Option 2] Create a public API for external crates to implement their own bindings.**

* Good, because ownership will be clear, and members of the community can opt to maintain their own binding generators.
* Good, because our CI would only need to test the core bindings we maintain, and others can be tested by their own maintainers (for example, a `gecko-js` binding generator should be tested in `mozilla-central` and not here).
* Good, because a release in external bindings wouldn't have an impact on any internal ones unless it changes internal `uniffi` behavior.
* Bad, because it's easier to accidentally have a version mismatch. (see [this ADR](https://github.com/mozilla/uniffi-rs/pull/1203))
* Bad, because testability increases in complexity. We are required to publish fixtures and examples we have. (see [PR 1206](https://github.com/mozilla/uniffi-rs/pull/1206))

Overall this option is preferred because:

- It's a requirement to implement bindings for gecko-js, which can't be tested end-to-end without a complex build system.
- It creates the possibility of community contributors writing and maintaining their own binding generators in their own repositories.
- The increased risk of version mismatch can be dealt with. (see [this ADR](https://github.com/mozilla/uniffi-rs/pull/1203))


## Decision Outcome

Chosen option:
### **[Option 2] Create a public API for external crates to implement their own bindings.**

## Changes
### Expose a trait `BindingGenerator`
The trait would have the following:
1. An associated type that implements `BindingGeneratorConfig`
    - `BindingGeneratorConfig` would be another trait, that binding generators can implement on their own configuration types. The purpose of this type is to carry any binding specific configuration parsed from the `uniffi.toml`
1. A function `write_bindings` that takes in the ComponentInterface and Config and writes the bindings into directory `out_dir`

### Expose a generic function as entry point
1. The binding generator should call a generic function when generating bindings exposed by `uniffi_bindgen`. The generic function will:
    - Parse the UDL.
    - Parse the configuration from `uniffi.toml`, using the `BindingGeneratorConfig` trait the consumer implements.
    - Initialize a `BindingGenerator`, with the type a consumer provides.
    - Call `write_bindings` on the generic type.

See [PR 1201](https://github.com/mozilla/uniffi-rs/pull/1201) for implementation of the above changes.

### Expose fixtures for testings
To enable external binding generators to implement tests, we would publish our fixtures and a new `uniffi_testing` crate that is a helper for consumers to build and consume the fixture crates.

See [PR 1206](https://github.com/mozilla/uniffi-rs/pull/1206) for implementation of the testing changes.
