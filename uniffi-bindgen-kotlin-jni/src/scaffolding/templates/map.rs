{%- let type_name = map.self_type.type_rs %}
{%- let lift_from_parts = "lift_from_parts_{}"|format(map.self_type.id) %}
{%- let lower_into_parts = "lower_into_parts_{}"|format(map.self_type.id) %}

unsafe fn {{ lower_into_parts }}(
    value: {{ type_name }},
) -> uniffi::Result<(*mut ::std::primitive::u8, ::std::primitive::usize)> {
    unsafe {
        let capacity = value.len() * {{ map.item_size }};
        let ptr = uniffi::ffibuffer::alloc(capacity)?;
        let mut pos = ptr;
        for (k, v) in value {
            {{ map.key.write_fn_rs() }}(pos, k)?;
            {{ map.value.write_fn_rs() }}(pos.add({{ map.value_offset }}), v)?;
            pos = pos.add({{ map.item_size }});
        }
        uniffi::Result::Ok((ptr, capacity))
    }
}

unsafe fn {{ lift_from_parts }}(
    ptr: *mut ::std::primitive::u8,
    capacity: ::std::primitive::usize,
) -> uniffi::Result<{{ type_name }}> {
    let mut do_lift = || {
        unsafe {
            let mut pos = ptr;
            let length = capacity / {{ map.item_size }};
            let mut map = {{ type_name }}::with_capacity(length);
            for _ in 0..length {
                map.insert(
                    {{ map.key.read_fn_rs() }}(pos)?,
                    {{ map.value.read_fn_rs() }}(pos.add({{ map.value_offset }}))?,
                );
                pos = pos.add({{ map.item_size }});
            }
            uniffi::Result::Ok(map)
        }
    };
    let result = do_lift();
    uniffi::ffibuffer::free(ptr, capacity);
    result
}

unsafe fn {{ map.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: {{ type_name }},
) -> uniffi::Result<uniffi_jni::jobject> {
    let (ptr, capacity) = {{ lower_into_parts }}(value)?;
    uniffi_jni::lower_buffer(uniffi_env, ptr, capacity)
}

unsafe fn {{ map.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    byte_buffer: uniffi_jni::jobject,
) -> uniffi::Result<{{ type_name }}> {
    let (ptr, capacity) = uniffi_jni::lift_buffer(uniffi_env, byte_buffer)?;
    {{ lift_from_parts }}(ptr, capacity)
}

unsafe fn {{ map.self_type.write_fn_rs() }}(
    buf_ptr: *mut ::std::primitive::u8,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    let (ptr, capacity) = {{ lower_into_parts }}(value)?;
    uniffi::ffibuffer::write_buffer(buf_ptr, ptr, capacity)?;
    uniffi::Result::Ok(())
}

unsafe fn {{ map.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    let (ptr, capacity) = uniffi::ffibuffer::read_buffer(ptr)?;
    {{ lift_from_parts }}(ptr, capacity)
}
