#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ jni_method_name }}(
    _uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
) {
    uniffi::trace!("Calling {{ callable.name }}");
    {{ callable.fully_qualified_name_rs }}();
}
