# UniFFI

UniFFI is a tool that automatically generates foreign-language bindings targeting Rust libraries.  
It fits in the practice of consolidating business logic in a single Rust library while targeting multiple platforms, making it simpler to develop and maintain a cross-platform codebase.  
Note that this tool will not help you ship a Rust library to these platforms, but simply not have to write bindings code by hand [[0]](https://i.kym-cdn.com/photos/images/newsfeed/000/572/078/d6d.jpg).

## Design

UniFFI requires you to declare the interface you want to expose to other languages using a restricted
subset of Rust syntax, along with a couple of helper macros. This interface declaration is used
to generate two things:

* Alongside your hand-written Rust code, UniFFI will generate some Rust *scaffolding* that exposes
your Rust datatypes and functions over a low-level C-compatible FFI.
* For each target foreign language, UniFFI will use the interface declaration to generate
*foreign-language bindings* that consume this low-level FFI and expose it via more idiomatic
higher-level code in the target language.

![uniffi diagram](./uniffi_diagram.png)

## Supported languages

- Kotlin
- Swift
- Python
- Ruby
