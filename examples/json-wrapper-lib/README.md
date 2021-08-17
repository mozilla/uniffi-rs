# json-wrapper-lib

This is an example of an library that uses an type from an extension crate.
JsonObject, defined in json-wrapper-ext, gets sent across the FFI boundary as a
String and leverages the existing String FFIConverter code.
