{%- let type_name = custom.self_type.type_rs %}

unsafe fn {{ custom.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<{{ custom.builtin.lowered_type_rs() }}> {
    unsafe {
        let builtin_value = <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::lower(value);
        {{ custom.builtin.lower_fn_rs() }}(uniffi_env, builtin_value)
    }
}

unsafe fn {{ custom.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    {%- for ffi_type in custom.builtin.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        let builtin_value = {{ custom.builtin.lift_fn_rs() }}(
            uniffi_env,
            {%- for _ in custom.builtin.ffi_types %}
            v{{ loop.index0 }},
            {%- endfor %}
        )?;
        <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::try_lift(builtin_value)
    }
}

unsafe fn {{ custom.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    unsafe {
        let builtin_value = <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::lower(value);
        {{ custom.builtin.write_fn_rs() }}(ptr, builtin_value)?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ custom.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        let builtin_value = {{ custom.builtin.read_fn_rs() }}(ptr)?;
        <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::try_lift(builtin_value)
    }
        
}

