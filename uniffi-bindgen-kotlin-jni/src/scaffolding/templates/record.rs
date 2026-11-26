{%- let type_name = rec.self_type.type_rs %}

{%- if rec.passed_as_buffer %}
// {{ rec.name }} deconstructs into too many FFI values, so it's passed as a FFI buffer
unsafe fn {{ rec.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    rec: {{ type_name }},
) -> uniffi::Result<uniffi_jni::jobject> {
    unsafe {
        let capacity = {{ rec.buffer_size }};
        let ptr = uniffi::ffibuffer::alloc(capacity)?;
        {{ rec.self_type.write_fn_rs() }}(ptr, rec)?;
        uniffi_jni::lower_buffer(uniffi_env, ptr, capacity)
    }
}

unsafe fn {{ rec.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    byte_buffer: uniffi_jni::jobject,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        let (ptr, capacity) = uniffi_jni::lift_buffer(uniffi_env, byte_buffer)?;
        let result = {{ rec.self_type.read_fn_rs() }}(ptr);
        uniffi::ffibuffer::free(ptr, capacity);
        result
    }
}
{%- else %}
unsafe fn {{ rec.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    rec: {{ type_name }},
) -> uniffi::Result<{{ rec.self_type.lowered_type_rs() }}> {
    unsafe {
        {%- for field in rec.fields %}
        let uniffi_field_lowered_{{ field.index }} = {{ field.ty.lower_fn_rs() }}(uniffi_env, rec.{{ field.name_rs() }})?;
        {%- endfor %}

        {%- if rec.self_type.lowers_to_primitive() %}
        uniffi::Result::Ok(uniffi_field_lowered_{{ rec.primitive_field_index() }})
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
{%- endif %}

unsafe fn {{ rec.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    unsafe {
        {%- for f in rec.fields %}
        {{ f.ty.write_fn_rs() }}(ptr.add({{ f.offset }}), value.{{ f.name_rs() }})?;
        {%- endfor %}
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ rec.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        uniffi::Result::Ok({{ type_name }} {
            {%- for f in rec.fields %}
            {{ f.name_rs() }}: {{ f.ty.read_fn_rs() }}(ptr.add({{ f.offset }}))?,
            {%- endfor %}
        })
    }
}
