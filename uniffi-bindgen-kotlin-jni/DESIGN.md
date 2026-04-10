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

# Errors/exceptions

Errors/exceptions are handled JNI rather than `uniffi::RustCallStatus`

For expected errors, we call a generated function in the `uniffi` package
that's responsible for constructing the error and throwing.

The throw function is passed an FFI buffer argument to read from.
The callee passes the same FFI buffer that the caller passed to them.
The caller is still responsible for freeing the buffer in the case of an exception.

For unexpected errors, we construct a `uniffi.InternalException` instance and throw it using JNI.
This is a simpler case since we can use the JNI `ThrowNew` function to construct
the exception from a message string.

# Kotlin `uniffi` package

This is a generated package that contains all the FFI functions.
Putting these functions here has some advantages over our current system:

* **Less namespace pollution**.
  With `uniffi-bindgen-kotlin` have to add public functions for UniFFI-internal operations.
  These functions need to be public to make external types work.
  However, it feels slightly wrong for them to end up in the consumer-facing package.
  Putting these functions in the `uniffi` package feels better.
* **Less duplication**.
  If multiple crates use `Vec<u8>` in their interfaces, we'll only need to define 1 read and 1 write function.
* **Simplified external types**.
  We don't need to map namespaces to package names to find the right function.

# Custom types

We use the `uniffi::CustomType` trait to handle this.
The `custom_type!` macro implements it for the custom type.

Note: this is one place where wheree we need the macros to generate code.
This is because the `into` and `try_from` expressions need to be executed
where they're defined rather than in the crate where we're generating scaffolding.

# Callback interfaces

Callabck interfaces are implemented by defining a Kotlin dispatch method
for each callback interface method.
This is responsible for finding the object in the handle map,
reading/converting all arguments from the FFI buffer,
calling the real method,
then writing the return value back to the FFI buffer.
We don't need an `init` function or a vtable,
since we can just lookup and call the methods directly from JNI.

Callback interfaces have essentially the same FFI as Rust functions,
except the calling direction is reversed and they also input the callback object handle.

In the future, we could explore more radical ideas,
like storing the callback object pointer directly in Rust.

# Trait interfaces

Trait interfaces are passed across the FFI using 2 64-bit handles.
For Rust-implemented traits, this is casted from the `Arc<dyn Trait>` wide pointer.
For Kotlin-implemented traits, the first handle is always `0`
and the second is the callback interface handle.

This avoids creating extra Box wrapper that we currently create and also gives us a simple way
to check which side of the FFI implements a trait.

# JNI

This crate uses the low-level `jni_sys` crate rather than the high-level `jni` crate.
This allows for more control and better performance in the JNI code.
An earlier version used `jni`, but benchmarks showed something like ~2-4x increase when switching to `jni_sys`.
`jni_sys` is not much harder for us to use, since we're operating at a very low level.

`jni_sys` is also a more light-weight dependency,
which is important since we're forcing all consumer crates to depend on it.
