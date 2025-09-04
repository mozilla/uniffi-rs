# UniFFI

UniFFI is a tool that automatically generates foreign-language bindings targeting Rust libraries.
The repository can be found on [github](https://github.com/mozilla/uniffi-rs/).
It fits in the practice of consolidating business logic in a single Rust library while targeting multiple platforms, making it simpler to develop and maintain a cross-platform codebase.
Note that this tool will not help you ship a Rust library to these platforms, but it will help you avoid writing bindings code by hand.
[Related](https://i.kym-cdn.com/photos/images/newsfeed/000/572/078/d6d.jpg).

## Design

UniFFI requires you to describe your interface via either [proc-macros](./proc_macro/index.md) or in an [Interface Definition Language](./udl/index.md) (based on [WebIDL](https://webidl.spec.whatwg.org/)) file.
These definitions describe the methods and data structures available to the targeted languages, and are used to generate Rust *scaffolding* code and foreign-language *bindings*.
This process can take place either during the build process or be manually initiated by the developer.

![uniffi diagram](./uniffi_diagram.png)

## Supported languages

UniFFI comes with full support for Kotlin, Swift and Python; unless specified otherwise, you can expect all features in
this manual will work for these languages.

We also have partial legacy support for Ruby; the UniFFI team keeps the existing Ruby support working but tends to not
add new features to that language. It seems possible that Ruby support will be split into its own crate at some point, but
in the meantime we welcome improvements and contributions to Ruby.

There are also many 3rd party bindings - please see our [README](https://github.com/mozilla/uniffi-rs/blob/main/README.md) for references.
These languages may require older versions of UniFFI and may have partial or non-existant support for some features; see the
documentation for those bindings for details.