{%- let type_name = box_.self_type.type_rs %}
{%- let lift_from_handle = "lift_from_handle{}"|format(box_.self_type.id) %}
{%- let lower_into_handle = "lower_into_handle{}"|format(box_.self_type.id) %}

unsafe fn {{ lower_into_handle }}(
    value: {{ type_name }},
) -> ::std::primitive::i64 {
    unsafe {
        let raw_ptr = ::std::boxed::Box::into_raw(value);
        uniffi::trace!("lower box: {raw_ptr:?}");
        raw_ptr.expose_provenance() as ::std::primitive::i64
    }
}

unsafe fn {{ lift_from_handle }}(
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
    uniffi::Result::Ok({{ lower_into_handle }}(value))
}

unsafe fn {{ box_.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    handle: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok({{ lift_from_handle }}(handle))
}

unsafe fn {{ box_.self_type.read_fn_rs() }}(
    buf_ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok({{ lift_from_handle }}(uniffi::ffibuffer::read_i64(buf_ptr)?))
}

unsafe fn {{ box_.self_type.write_fn_rs() }}(
    buf_ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    uniffi::ffibuffer::write_i64(buf_ptr, {{ lower_into_handle }}(value))?;
    uniffi::Result::Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ box_.jni_from_ffi_values_name() }}(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    {%- for ffi_type in box_.inner.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) -> ::std::primitive::i64 {
    uniffi_jni::rust_call_with_env(env, |env| {
        let b = ::std::boxed::Box::new({{ box_.inner.lift_fn_rs() }}(
            env,
            {%- for _ in box_.inner.ffi_types %}
            v{{ loop.index0 }},
            {%- endfor %}
        )?);
        uniffi::Result::Ok({{ lower_into_handle }}(b))
    })
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ box_.jni_into_inner_name() }}(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: ::std::primitive::i64,
) -> uniffi_jni::jobject {
    uniffi_jni::rust_call_with_env(env, |env| {
        let b = {{ lift_from_handle }}(handle);
        let lowered = {{ box_.inner.lower_fn_rs() }}(env, *b)?;
        {{ box_.inner.lift_kt_from_rust_var() }}.call_object(
            env,
            [
                {%- for (var, ffi_type) in box_.inner.ffi_values_rs("lowered") %}
                uniffi_jni::jvalue {
                    {{ ffi_type.jvalue_field() }}: {{ var }},
                },
                {%- endfor %}
            ]
        ).to_anyhow_result(env, "{{ box_.inner.lift_fn_kt() }}")
    })
}
