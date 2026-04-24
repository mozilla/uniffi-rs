/// Create a new `{{ type_node.type_kt }}` instance
///
/// # Safety
/// `ffi_buffer` must point to a valid FFI buffer
unsafe fn {{ type_node.throw_error_fn_rs() }}(
    env: *mut uniffi_jni::JNIEnv,
    ffi_buffer: *mut u8,
) {
    static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
        c"uniffi/UniffiKt",
        c"{{ type_node.throw_error_fn_kt() }}",
        c"(J)V",
    );
    // Safety:
    // We're using the JNI API correctly.
    unsafe {
        // Exceptions are expected, they'll be thrown when the native method returns.
        let _ = METHOD.call_void(env, [
            uniffi_jni::jvalue {
                j: ffi_buffer.expose_provenance() as i64,
            }
        ]);
    }
}
