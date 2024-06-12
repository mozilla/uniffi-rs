This directory contains the tests for external types - cross-crate depedencies and libraries
to tie them together.

- `lib` is a library crate that depends on various other crates.
- `proc-macro-lib` is another library crate, but this one uses proc-macros rather than UDL files.

The various other crates all inter-relate and are ultimately consumed by the above libs:
- `custom-types` is all about wrapping types (eg, Guid, Handle) in a native type (eg, String, u64)
- `uniffi-one` is just a normal other crate also using uniffi.
- `sub-lib` itself consumes and exposes the other types.
- `external-crate` doesn't depend on uniffi but has types we expose.

etc.
