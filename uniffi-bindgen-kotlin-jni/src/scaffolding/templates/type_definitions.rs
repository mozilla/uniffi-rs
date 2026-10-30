{%- for type_def in package.type_definitions %}
{%- match type_def %}
{%- when TypeDefinition::Class(cls) %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_{{ package.jni_symbol_name() }}_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    {%- if !cls.imp.is_trait_interface() %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    {%- else %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(handle as usize);
    {%- endif %}
    uniffi::trace!("free object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_{{ package.jni_symbol_name() }}_uniffi_Scaffolding_{{ cls.jni_clone_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) -> i64 {
    {%- if !cls.imp.is_trait_interface() %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    {%- else %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(handle as usize);
    {%- endif %}
    uniffi::trace!("clone object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
    // Return the handle back now that the ref count has been incremented.
    handle
}
{%- when TypeDefinition::Box(box_) %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_{{ package.jni_symbol_name() }}_uniffi_Scaffolding_{{ box_.jni_from_ffi_values_name() }}(
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
        uniffi::Result::Ok({{ box_.lower_into_handle_fn_rs() }}(b))
    })
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_{{ package.jni_symbol_name() }}_uniffi_Scaffolding_{{ box_.jni_into_inner_name() }}(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: ::std::primitive::i64,
) -> uniffi_jni::jobject {
    uniffi_jni::rust_call_with_env(env, |env| {
        let b = {{ box_.lift_from_handle_fn_rs() }}(handle);
        let lowered = {{ box_.inner.lower_fn_rs() }}(env, *b)?;
        {{ package.lift_kt_from_rust_var(box_.inner) }}.call_object(
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
{%- else %}
{%- endmatch %}
{%- endfor %}
