{%- let type_name = cls.self_type.type_rs %}
{%- let inner_type_name = "{}::{}"|format(cls.module_path, cls.name_rs()) %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ cls.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let raw_ptr = cursor.read_ptr::<{{ inner_type_name }}>()?;
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        Ok(::std::sync::Arc::from_raw(raw_ptr))
    }
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
///
/// Inputs ArcOrOwned<T> so that it's compatible with both `T` and `Arc<T>`.
pub fn {{ cls.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: impl uniffi::ArcOrOwned<{{ inner_type_name }}>,
) -> uniffi::Result<()> {
    let raw_ptr = ::std::sync::Arc::into_raw(value.into_arc());
    cursor.write_ptr::<{{ inner_type_name }}>(raw_ptr.cast_mut())?;
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ inner_type_name }}>(handle as usize);
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_addref_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ inner_type_name }}>(handle as usize);
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
}
