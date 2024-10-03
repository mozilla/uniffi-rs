# Swift Bindings

UniFFI ships with production-quality support for generating Swift bindings.
Concepts from the UDL file map into Swift as follows:

* Primitive datatypes map to their obvious Swift counterpart, e.g. `u32` becomes `UInt32`,
  `string` becomes `String`, `bytes` becomes `Data`, etc.
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

## Generated files

UniFFI generates several kinds of files for Swift bindings:

* C header files declaring the FFI structs/functions used by the Rust scaffolding code
* A modulemap, which defines a Swift module for the C FFI definitions in the header file.
* A Swift source file that defines the Swift API used by consumers.  This imports the FFI module.
