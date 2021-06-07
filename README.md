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

## User Guide

You can read more about using the tool in [**the UniFFI user guide**](https://mozilla.github.io/uniffi-rs/).

Please be aware that UniFFI is being developed concurrently with its initial consumers, so it is changing rapidly and there
are a number of sharp edges to the user experience. Still, we consider is developed enough for production use in Mozilla
products and we welcome any feedback you may have about making it more broadly useful.


## Contributing

If this tool sounds interesting to you, please help us develop it! You can:

* View the [contributor guidelines](./docs/contributing.md).
* File or work on [issues](https://github.com/mozilla/uniffi-rs/issues) here in GitHub.
* Join discussions in the [Cross-Platform Rust Components](https://chat.mozilla.org/#/room/#rust-components:mozilla.org)
  room on Matrix.

## Code of Conduct

This project is governed by Mozilla's [Community Participation Guidelines](./CODE_OF_CONDUCT.md).

---

(Versions `v0.9.0` though `v0.11.0` include a deprecation notice that links to this README. Once those versions have
sufficiently aged out, this section can be removed from the top-level README.)

### Thread Safety

It is your responsibility to ensure the structs you expose via UniFFI are
all `Send+Sync`. This will be enforced by the Rust compiler, likely with an
inscrutable error from somewhere in UniFFI's generated Rust code.

Early versions of this crate automatically wrapped rust structs in a mutex,
thus implicitly making the interfaces thread-safe and safe to be called
over the FFI by any thread. However, in practice we found this to be a
mis-feature, so version 0.7 first introduced the ability for the component
author to opt-out of this implicit wrapping and take care of thread-safety
themselves by adding a `[Threadsafe]` attribute to the interface.

Version 0.9.0 took this further, and interfaces not marked as `[Threadsafe]` 
started issuing a deprecation warning. If you are seeing these deprecation warnings,
you should upgrade your component as soon as possible. For an example of what kind of
effort is required to make your interfaces thread-safe, you might like to see
[this commit](https://github.com/mozilla/uniffi-rs/commit/454dfff6aa560dffad980a9258853108a44d5985)
where we made one the examples thread-safe.

As of version 0.11.0, all interfaces will be required to be  `Send+Sync`, and the
`[Threadsafe]` attribute will be deprecated and ignored.

See also [adr-0004](https://github.com/mozilla/uniffi-rs/blob/main/docs/adr/0004-only-threadsafe-interfaces.md)
which outlines the reasoning behind this decision.