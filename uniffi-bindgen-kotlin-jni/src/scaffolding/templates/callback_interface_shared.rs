{%- for result in root.kotlin_sync_callable_results() %}
{%- if let Some(return_type) = result.return_type %}
{%- if !return_type.lowers_to_primitive() %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ result.set_callback_return_fn() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    result_handle: i64,
    {%- for ffi_type in return_type.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) {
    unsafe {
        let result_pointer = ::std::ptr::with_exposed_provenance_mut::<::std::option::Option<{{ result.return_type_rs() }}>>(result_handle as usize);
        match {{ return_type.lift_fn_rs() }}(
            uniffi_env,
            {%- for _ in return_type.ffi_types %}
            v{{ loop.index0 }},
            {%- endfor %}
        ) {
            Ok(return_value) => {
                {%- if result.throws_type.is_none() %}
                result_pointer.write(::std::option::Option::Some(return_value));
                {%- else %}
                result_pointer.write(::std::option::Option::Some(::std::result::Result::Ok(return_value)));
                {%- endif %}
            }
            Err(e) => {
                uniffi_jni::throw_internal_exception(uniffi_env, format!("{{ result.set_callback_return_fn() }} failed: {e}").into());
            }
        }
    }
}
{%- endif %}
{%- endif %}

{%- if let Some(throws_type) = result.throws_type %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ result.set_callback_err_fn() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    result_handle: i64,
    {%- for ffi_type in throws_type.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) {
    unsafe {
        let result_pointer = ::std::ptr::with_exposed_provenance_mut::<::std::option::Option<{{ result.return_type_rs() }}>>(
            result_handle as usize,
        );
        match {{ throws_type.lift_fn_rs() }}(
            uniffi_env,
            {%- for _ in throws_type.ffi_types %}
            v{{ loop.index0 }},
            {%- endfor %}
        ) {
            Ok(return_value) => {
                result_pointer.write(::std::option::Option::Some(::std::result::Result::Err(return_value)));
            }
            Err(e) => {
                uniffi_jni::throw_internal_exception(uniffi_env, format!("{{ result.set_callback_return_fn() }} failed: {e}").into());
            }
        }
    }
}
{%- endif %}

{%- endfor %}
