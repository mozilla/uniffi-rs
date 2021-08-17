# json-wrapper-ext

This is an example of an extension crate which defines a JsonObject type that
can be used with UniFFI.  JsonObject gets sent across the FFI boundary as a
String and leverages the existing String FFIConverter code.
