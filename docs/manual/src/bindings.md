# Generating bindings

Bindings is the term used for the code generates for foreign languages which integrate
with Rust crates - that is, the generated Python, Swift or Kotlin code which drives the
examples.

UniFFI comes with a `uniffi_bindgen` which generates these bindings. For introductory
information, see [Foreign Language Bindings in the tutorial](./tutorial/foreign_language_bindings.md)

# Customizing the binding generation.

Each of the bindings reads a file `uniffi.toml` in the root of a crate which supports
various options which influence how the bindings are generated. Default options will be used
if this file is missing.

Each binding supports different options, so please see the documentation for each binding language.
