`uniffi-bindgen-kotlin-jni` is an experimental bindgen system that generates
JNI-based Rust scaffolding and Kotlin bindings to call into it.

# Code generation

`uniffi-bindgen-kotin-jni` uses `uniffi_parse_rs` to parse the Rust source
and create the metadata needed to generate the bindings/scaffolding.

The scaffolding is generated using the `uniffi_pipeline` framework and Askama templates
rather than from the macros.
We avoided generating this via the macros,
since it's expected that some external bindgens will also want language-specific scaffolding
and there's no clear way for them to hook into `uniffi_macros` to do that.

Scaffolding is generated for a single crate only.
The main reason is that we only want to parse the source code once.

# FFI calling convention

`uniffi-bindgen-kotin-jni` leverages the `uniffi::FfiBuffer` type to pass values across the FFI.
All arguments and return values are written/read from the buffer.

* The caller is responsible for allocating and freeing the buffer.
* The caller passes the buffer to the callee.
* If there is a return value, the callee writes it to the same buffer.
