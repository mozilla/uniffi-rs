static UNIFFI_GLOBAL_JVM: ::std::sync::atomic::AtomicPtr<uniffi_jni::JavaVM> =
    ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut());

#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: *mut uniffi_jni::JavaVM, _reserved: *mut ()) -> uniffi_jni::jint {
    UNIFFI_GLOBAL_JVM.store(vm, ::std::sync::atomic::Ordering::Relaxed);
    uniffi_jni::JNI_VERSION_1_2
}

fn uniffi_get_global_jvm() -> *mut uniffi_jni::JavaVM {
    let jvm = UNIFFI_GLOBAL_JVM.load(::std::sync::atomic::Ordering::Relaxed);
    if jvm.is_null() {
        panic!("Global JavaVM not set, is Java running?");
    }
    jvm
}
