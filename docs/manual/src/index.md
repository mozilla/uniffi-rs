# UniFFI

UniFFI is a tool that automatically generates foreign-language bindings targeting Rust libraries.
The repository can be found on [github](https://github.com/mozilla/uniffi-rs/).
It fits in the practice of consolidating business logic in a single Rust library while targeting multiple platforms, making it simpler to develop and maintain a cross-platform codebase.
Note that this tool will not help you ship a Rust library to these platforms, but it will help you avoid writing bindings code by hand.
[Related](https://i.kym-cdn.com/photos/images/newsfeed/000/572/078/d6d.jpg).

## Design

UniFFI requires you to describe your interface via either proc-macros or in an Interface Definition Language (based on [WebIDL](https://webidl.spec.whatwg.org/)) file.
These definitions describe the methods and data structures available to the targeted languages, and are used to generate Rust *scaffolding* code and foreign-language *bindings*.
This process can take place either during the build process or be manually initiated by the developer.

![uniffi diagram](./uniffi_diagram.png)

## Supported languages

- Kotlin
- Swift
- Python
- Ruby

## Third-party foreign language bindings

* [Kotlin Multiplatform](https://gitlab.com/trixnity/uniffi-kotlin-multiplatform-bindings)
* [Go bindings](https://github.com/NordSecurity/uniffi-bindgen-go)
* [C# bindings](https://github.com/NordSecurity/uniffi-bindgen-cs)
