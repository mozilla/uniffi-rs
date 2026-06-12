# Passes

`uniffi_parse_rs` is designed to parse Rust source code in 2 passes:

The first pass parses the Rust syntax to create an intermediate representation (IR).
This pass focuses on syn parsing and only applies local reasoning.
For example, `syn::Type` is not parsed, since that would require resolving the type path
to find the struct/enum with a UniFFI derive.
Failable functions for this pass generally return `syn::Result`,
which simplifies things since we're often calling syn functions at this point.

The second pass converts the IR from the first pass into `uniffi_meta` types.
The main task here is converting `syn::Type` into `uniffi_meta::Type`,
which requires resolving the type paths (e.g. determining that `foo::bar::Baz` is a Record).
Failable functions for this pass generally return `crate::Result`,
which allows us to report better errors by tracking path resolution context.
For example, if a type alias is involved, then we want to also show the user
that the error happened when evaluating that type aliases.

This split has several advantages:

* It's easier to resolve types after performing the first pass.
* The code for the first pass can be integrated with `uniffi_macros`
  since it only parses one item at a time and it uses `syn::Result`.

# Converting to uniffi_meta

Converting the IR from the second pass to `uniffi_meta` items is done on demand.
We only try to resolve a `syn::Type` into a `uniffi_meta::Type` when creating
the `uniffi_meta::Metadata` item that contains that type.
This allows us to ignore types in the Rust source that we don't need to know about.

# Parsing items

We always parse the attributes first to make sure we're using the correct parser.
For example, when parsing a function in an `impl` block,
we parse the attributes and look for `uniffi::constructor`.
This way we know if we should expect a self argument or not.

# Inputs

This crate does not know how to find source files, it expects them to be passed in.
Also, although it handles conditional compilation, it doesn't handle feature detection.
Both of those things are handled by `uniffi_bindgen`, which parses the cargo metadata.
