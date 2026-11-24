{%- let type_name = box_.self_type.type_rs %}

unsafe fn {{ box_.lower_into_handle_fn_rs() }}(
    value: {{ type_name }},
) -> ::std::primitive::i64 {
    unsafe {
        let raw_ptr = ::std::boxed::Box::into_raw(value);
        uniffi::trace!("lower box: {raw_ptr:?}");
        raw_ptr.expose_provenance() as ::std::primitive::i64
    }
}

unsafe fn {{ box_.lift_from_handle_fn_rs() }}(
    handle: ::std::primitive::i64,
) -> {{ type_name }} {
    unsafe {
        let raw_ptr = ::std::ptr::with_exposed_provenance_mut(handle as ::std::primitive::usize);
        uniffi::trace!("lift box: {raw_ptr:?}");
        ::std::boxed::Box::from_raw(raw_ptr)
    }
}

unsafe fn {{ box_.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<::std::primitive::i64> {
    uniffi::Result::Ok({{ box_.lower_into_handle_fn_rs() }}(value))
}

unsafe fn {{ box_.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    handle: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok({{ box_.lift_from_handle_fn_rs() }}(handle))
}

unsafe fn {{ box_.self_type.read_fn_rs() }}(
    buf_ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok({{ box_.lift_from_handle_fn_rs() }}(uniffi::ffibuffer::read_i64(buf_ptr)?))
}

unsafe fn {{ box_.self_type.write_fn_rs() }}(
    buf_ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    uniffi::ffibuffer::write_i64(buf_ptr, {{ box_.lower_into_handle_fn_rs() }}(value))?;
    uniffi::Result::Ok(())
}
