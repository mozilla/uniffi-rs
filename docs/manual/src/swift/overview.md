# Swift Bindings

UniFFI ships with production-quality support for generating Swift bindings.
Concepts from the UDL file map into Swift as follows:

* Primitive datatypes map to their obvious Swift counterpart, e.g. `u32` becomes `UInt32`,
  `string` becomes `String`, etc.
* An object interface declared as `interface T` is represented as a Swift `protocol TProtocol`
  and a concrete Swift `class T` that conforms to it. Having the protocol declared explicitly
  can be useful for mocking instances of the class in unittests.
* A dictionary struct declared as `dictionary T` is represented as a Swift `struct T`
  with public mutable fields.
* An enum declared `enum T` or `[Enum] interface T` is represented as a Swift
  `enum T` with appropriate variants.
* Optional types are represented using Swift's builtin optional type syntax `T?`.
* Sequences are represented as Swift arrays, and Maps as Swift dictionaries.
* Errors are represented as Swift enums that conform to the `Error` protocol.
* Function calls that have an associated error type are marked with `throws`,
  and hence must be called using one of Swift's `try` syntax variants.
* Failing assertions, Rust panics, and other unexpected errors in the generated code
  are translated into a private enum conforming to the `Error` protocol.
    * If this happens inside a throwing Swift function, it can be caught and handled
      by a catch-all `catch` statement (but do so at your own risk, because it indicates
      that something has gone seriously wrong).
    * If this happens inside a non-throwing Swift function, it will be converted
      into a fatal Swift error that cannot be caught.

Conceptually, the generated bindings are split into two Swift modules, one for the low-level
C FFI layer and one for the higher-level Swift bindings. For a UniFFI component named "example"
we generate:

* A C header file `exampleFFI.h` declaring the low-level structs and functions for calling
  into Rust, along with a corresponding `exampleFFI.modulemap` to expose them to Swift.
* A Swift source file `example.swift` that imports the `exampleFFI` module and wraps it
  to provide the higher-level Swift API.

Splitting up the bindings in this way gives you flexibility over how both the Rust code
and the Swift code are distributed to consumers. For example, you may choose to compile
and distribute the Rust code for several UniFFI components as a single shared library
in order to reduce the compiled code size, while distributing their Swift wrappers as
individual modules.

For more technical details on how the bindings work internally, please see the
[module documentation](../internals/api/uniffi_bindgen/bindings/swift/index.html)
