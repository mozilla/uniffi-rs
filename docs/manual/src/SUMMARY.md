# Summary

[Overview](./Overview.md)
- [Motivation](./Motivation.md)
- [Tutorial](./Getting_started.md)
  - [Prerequisites](./tutorial/Prerequisites.md)
  - [Describing the interface](./tutorial/udl_file.md)
  - [Generating the Rust scaffolding code](./tutorial/Rust_scaffolding.md)
  - [Generating the foreign-language bindings](./tutorial/foreign_language_bindings.md)
- [The UDL file](./udl_file_spec.md)
  - [Namespace](./udl/namespace.md)
  - [Built-in types](./udl/builtin_types.md)
  - [Enumerations](./udl/enumerations.md)
  - [Structs/Dictionaries](./udl/structs.md)
  - [Functions](./udl/functions.md)
    - [Throwing errors](./udl/errors.md)
  - [Interfaces/Objects](./udl/interfaces.md)
  - [Callback Interfaces](./udl/callback_interfaces.md)
  - [External Types](./udl/ext_types.md)
    - [Declaring External Types](./udl/ext_types_external.md)
    - [Declaring Custom Types](./udl/custom_types.md)
- [Experimental: Attributes and Derives](./proc_macro/index.md)

# Kotlin

- [Integrating with Gradle](./kotlin/gradle.md)
- [Kotlin Lifetimes](./kotlin/lifetimes.md)

# Swift

- [Overview](./swift/overview.md)
- [Configuration](./swift/configuration.md)
- [Building a Swift module](./swift/module.md)
- [Integrating with Xcode](./swift/xcode.md)
- [Wrapped in Framework](./swift/framework.md)

# Internals
- [Design Principles](./internals/design_principles.md)
- [Navigating the Code](./internals/crates.md)
- [Lifting, Lowering, and Serialization](./internals/lifting_and_lowering.md)
- [Managing Object References](./internals/object_references.md)
- [Rendering Foreign Bindings](./internals/rendering_foreign_bindings.md)
