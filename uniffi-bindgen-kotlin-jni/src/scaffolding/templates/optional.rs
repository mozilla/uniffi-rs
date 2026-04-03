{%- let type_name = opt.self_type.type_rs %}

{%- if !opt.self_type.lowers_to_primitive() %}
{%- let all_ffi_types = opt.self_type.ffi_types %}
{%- let inner_ffi_types = opt.inner.ffi_types %}

unsafe fn {{ opt.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<{{ opt.self_type.lowered_type_rs() }}> {
    uniffi::Result::Ok(match value {
        ::std::option::Option::Some(v) => {
            let inner_lowered = {{ opt.inner.lower_fn_rs() }}(uniffi_env, v)?;
            (
                true,
                {%- for (var, _) in opt.inner.ffi_values_rs("inner_lowered") %}
                {{ var }},
                {%- endfor %}
            )
        }
        ::std::option::Option::None => {
            (
                false,
                {%- for ffi_type in inner_ffi_types %}
                <{{ ffi_type.type_rs() }} as ::std::default::Default>::default(),
                {%- endfor %}
            )
        }
    })
}

unsafe fn {{ opt.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    {%- for ffi_type in all_ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok(if v0 {
        ::std::option::Option::Some({{ opt.inner.lift_fn_rs() }}(
            uniffi_env,
            {%- for _ in inner_ffi_types %}
            v{{ loop.index0 + 1 }},
            {%- endfor %}
        )?)
    } else {
        ::std::option::Option::None
    })
}
{%- else if opt.inner.is_interface() %}

unsafe fn {{ opt.self_type.lower_fn_rs() }}(
    env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<::std::primitive::i64> {
    uniffi::Result::Ok(match value {
        ::std::option::Option::Some(v) => {{ opt.inner.lower_fn_rs() }}(env, v)?,
        ::std::option::Option::None => 0,
    })
}

unsafe fn {{ opt.self_type.lift_fn_rs() }}(
    env: *mut uniffi_jni::JNIEnv,
    handle: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    uniffi::Result::Ok(if handle != 0 {
        ::std::option::Option::Some({{ opt.inner.lift_fn_rs() }}(env, handle)?)
    } else {
        ::std::option::Option::None
    })
}

{%- endif %}

unsafe fn {{ opt.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    unsafe {
        match value {
            None => {
                uniffi::ffibuffer::write_bool(ptr, false)?;
            }
            Some(inner_value) => {
                uniffi::ffibuffer::write_bool(ptr, true)?;
                {{ opt.inner.write_fn_rs() }}(ptr.add({{ opt.some_offset }}), inner_value)?;
            }
        }
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ opt.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        uniffi::Result::Ok(match uniffi::ffibuffer::read_bool(ptr)? {
            false => {
                None
            }
            true => {
                Some({{ opt.inner.read_fn_rs() }}(ptr.add({{ opt.some_offset }}))?)
            },
            n => uniffi::deps::anyhow::bail!("{{ opt.self_type.read_fn_rs() }}: invalid discriminent: {n}"),
        })
    }
}
