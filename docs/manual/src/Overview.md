# UniFFI

UniFFI is a tool that automatically generates foreign-language bindings targeting Rust libraries.
The repository can be found on [github](https://github.com/mozilla/uniffi-rs/).
It fits in the practice of consolidating business logic in a single Rust library while targeting multiple platforms, making it simpler to develop and maintain a cross-platform codebase.  
Note that this tool will not help you ship a Rust library to these platforms, but simply not have to write bindings code by hand [[0]](https://i.kym-cdn.com/photos/images/newsfeed/000/572/078/d6d.jpg).

## Design

UniFFI requires to write an Interface Definition Language (based on [WebIDL](https://heycam.github.io/webidl/)) file describing the methods and data structures available to the targeted languages.  
This .udl (UniFFI Definition Language) file, whose definitions must match with the exposed Rust code, is then used to generate Rust *scaffolding* code and foreign-languages *bindings*. This process can take place either during the build process or be manually initiated by the developer.

![uniffi diagram](./uniffi_diagram.png)

## Supported languages

- Kotlin
- Swift
- Python
- Ruby
