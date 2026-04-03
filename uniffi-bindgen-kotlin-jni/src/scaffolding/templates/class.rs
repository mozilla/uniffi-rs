{%- let type_name = cls.self_type.type_rs %}
{%- let arc_into_handle = "arc_into_handle_{}"|format(cls.self_type.id) %}
{%- let handle_into_arc = "handle_into_arc_{}"|format(cls.self_type.id) %}
{%- let handle_into_ref = "handle_into_ref_{}"|format(cls.self_type.id) %}
{%- let lift_object_ref = "lift_object_ref_{}"|format(cls.self_type.id) %}

unsafe fn {{ arc_into_handle }}(
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
) -> ::std::primitive::i64 {
    let raw_ptr = ::std::sync::Arc::into_raw(value.into_arc());
    uniffi::trace!("lower object: {raw_ptr:?}");
    raw_ptr as ::std::primitive::i64
}

unsafe fn {{ handle_into_arc }}(
    value: ::std::primitive::i64,
) -> {{ type_name }} {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut(value as usize);
    uniffi::trace!("lift object: {raw_ptr:?}");
    ::std::sync::Arc::from_raw(raw_ptr)
}

unsafe fn {{ handle_into_ref }}<'a>(
    value: ::std::primitive::i64,
) -> &'a {{ cls.inner_type_name() }} {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut(value as usize);
    uniffi::trace!("lift ref: {raw_ptr:?}");
    &*raw_ptr
}

unsafe fn {{ cls.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
) -> uniffi::Result<::std::primitive::i64> {
    unsafe { uniffi::Result::Ok({{ arc_into_handle }}(value)) }
}

unsafe fn {{ cls.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    unsafe { uniffi::Result::Ok({{ handle_into_arc }}(value)) }
}

/// Write a {{ type_name }} to a `FfiBuffer`
///
/// Inputs ArcOrOwned<T> so that it's compatible with both `T` and `Arc<T>`.
unsafe fn {{ cls.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
) -> uniffi::Result<()> {
    unsafe {
        let handle = {{ arc_into_handle }}(value);
        uniffi::ffibuffer::write_i64(ptr, handle)?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ cls.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        uniffi::Result::Ok({{ handle_into_arc }}(uniffi::ffibuffer::read_i64(ptr)?))
    }
}

unsafe fn {{ lift_object_ref }}<'a>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<&'a {{ cls.inner_type_name() }}> {
    unsafe { uniffi::Result::Ok({{ handle_into_ref }}(value)) }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    uniffi::trace!("free object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_clone_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) -> i64 {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    uniffi::trace!("clone object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
    // Return the handle back now that the ref count has been incremented.
    handle
}
