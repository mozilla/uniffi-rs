#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_{{ func.jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
) {
    uniffi_jni::rust_call(uniffi_env, |_| {
        {{ func.module_path }}::{{ func.callable.name_rs() }}();
    })
}
