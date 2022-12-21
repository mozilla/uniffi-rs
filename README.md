# UniFFI - a multi-language bindings generator for Rust

UniFFI is a toolkit for building cross-platform software components in Rust.

By writing your core business logic in Rust and describing its interface in a special
[interface definition file](https://mozilla.github.io/uniffi-rs/udl_file_spec.html),
you can use UniFFI to help you:

* Compile your Rust code into a shared library for use on different target platforms.
* Generate bindings to load and use the library from different target languages.

For example, UniFFI is currently used in the [mozilla/application-services](https://github.com/mozilla/application-services)
project to build browser storage and syncing functionality for Firefox mobile browsers. Core functionality is
written once in Rust, and auto-generated bindings allow that functionality to be called from both Kotlin (for Android apps)
and Swift (for iOS apps).

Currently supported "foreign" languages include Kotlin, Swift, Python and Ruby.
In general, we'd welcome contributions of other foreign bindings, but we recommend you contact the team first -
see [the contributing section below](#contributing).

## User Guide

You can read more about using the tool in [**the UniFFI user guide**](https://mozilla.github.io/uniffi-rs/).

Please be aware that UniFFI is being developed concurrently with its initial consumers, so it is changing rapidly and there
are a number of sharp edges to the user experience. Still, we consider is developed enough for production use in Mozilla
products and we welcome any feedback you may have about making it more broadly useful.

### Etymology and Pronunciation

ˈjuːnɪfaɪ. Pronounced to rhyme with "unify".

A portmanteau word that also puns with "unify", to signify the joining of one codebase accessed from many languages.

uni - [Latin ūni-, from ūnus, one]
FFI - [Abbreviation, Foreign Function Interface]

## Alternative tools

Other tools we know of which try and solve a similarly shaped problem are:

* [Diplomat](https://github.com/rust-diplomat/diplomat/) - see our [writeup of
  the different approach taken by that tool](docs/diplomat-and-macros.md)

(Please open a PR if you think other tools should be listed!)

## External resources

There are a few third-party resources that make it easier to work with UniFFI:

* [Kotlin Multiplatform support](https://gitlab.com/trixnity/uniffi-kotlin-multiplatform-bindings). The repository contains Kotlin Multiplatform bindings generation for UniFFI, letting you target both JVM and Native.
* [Plugin support for `.udl` files](https://github.com/Lonami/uniffi-dl) for the IDEA platform ([*uniffi-dl* in the JetBrains marketplace](https://plugins.jetbrains.com/plugin/20527-uniffi-dl)). It provides syntax highlighting, code folding, code completion, reference resolution and navigation (among others features) for the [UniFFI Definition Language (UDL)](https://mozilla.github.io/uniffi-rs/).

(Please open a PR if you think other resources should be listed!)

## Contributing

If this tool sounds interesting to you, please help us develop it! You can:

* View the [contributor guidelines](./docs/contributing.md).
* File or work on [issues](https://github.com/mozilla/uniffi-rs/issues) here in GitHub.
* Join discussions in the [#uniffi:mozilla.org](https://matrix.to/#/#uniffi:mozilla.org)
  room on Matrix.

## Code of Conduct

This project is governed by Mozilla's [Community Participation Guidelines](./CODE_OF_CONDUCT.md).
