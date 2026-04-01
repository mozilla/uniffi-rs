{%- let type_name = rec.self_type.type_rs %}

unsafe fn {{ rec.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    rec: {{ type_name }},
) -> uniffi::Result<{{ rec.self_type.lowered_type_rs() }}> {
    unsafe {
        {%- for field in rec.fields %}
        let uniffi_field_lowered_{{ field.index }} = {{ field.ty.lower_fn_rs() }}(uniffi_env, rec.{{ field.name_rs() }})?;
        {%- endfor %}

        {%- if rec.self_type.lowers_to_primitive() %}
        uniffi::Result::Ok(uniffi_field_lowered_0)
        {%- else %}
        uniffi::Result::Ok((
            {%- for field in rec.fields %}
            {%- for (var, _) in field.ty.ffi_values_rs(format!("uniffi_field_lowered_{}", field.index)) %}
            {{ var }},
            {%- endfor %}
            {%- endfor %}
        ))
        {%- endif %}
    }
}

unsafe fn {{ rec.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    {%- for ffi_type in rec.ffi_types() %}
    v{{ loop.index0 }}: {{ ffi_type.type_rs() }},
    {%- endfor %}
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        uniffi::Result::Ok({{ type_name }} {
            {%- for field in rec.fields %}
            {{ field.name_rs() }}: {{ field.ty.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_field in field.ffi_fields %}
                v{{ ffi_field.index }},
                {%- endfor %}
            )?,
            {%- endfor %}
        })
    }
}
