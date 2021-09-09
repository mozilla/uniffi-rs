This crate demonstrates examples of of UniFFI extensions.  Extensions create
new available types in the .UDL file by wrapping/converting existing UniFFI
types.  For example:

- `JsonObject` wraps a `String` to provide a JSON data structure.
- `SerializedObject` wraps a `String` to allow Kotlin/Python objects to be
  stored as strings in Rust.
- `Identifier` converts a `i64` into a library's `Identifier` type.

Extensions are stored as `.toml` files in the `uniffi` directory in your crate's
root directory.
