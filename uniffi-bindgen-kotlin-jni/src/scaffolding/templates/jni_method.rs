#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
) {
    uniffi::trace!("Calling {{ callable.name }}");
    unsafe {
        uniffi_jni::rust_call_with_env(uniffi_env, |_| {
            {{ callable.fully_qualified_name_rs }}();
            uniffi::Result::Ok(())
        })
    }
}
