# External types

*Note: The facility described in this document is not yet available for all foreign language
bindings.*

UniFFI supports refering to types defined outside of the UDL file. These types must be
either:

1) A locally defined type which [wraps a UniFFI primitive type](./custom_types.md).
2) A "UniFFI compatible" type [in another crate](./ext_types_external.md)

Specifically, "UniFFI compatible" means either a type defined in `udl` in an external crate, or
a type defined in another crate that satisfies (1).

These types are all declared using a `typedef`, with attributes specifying how the type is
handled. See the links for details.
