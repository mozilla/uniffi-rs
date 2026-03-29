#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_{{ func.jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_buf_handle: i64,
) {
     uniffi_jni::rust_call(uniffi_env, |_| {
        // Safety:
        // We assume the Kotlin side of the FFI sent us a valid buffer handle with arguments
        // correctly serialized.
        unsafe {
            let mut uniffi_buf = uniffi::FfiBuffer::from_ptr(
                ::std::ptr::with_exposed_provenance_mut(uniffi_buf_handle as usize)
            );
            {%- if !func.callable.arguments.is_empty() %}
            let (
                {%- for arg in func.callable.arguments %}
                {{ arg.name_rs() }},
                {%- endfor %}
            ) = uniffi_buf.with_cursor(|uniffi_reader| Ok((
                {%- for arg in func.callable.arguments %}
                {{ arg.ty.read_fn_rs }}(uniffi_reader)?,
                {%- endfor %}
            )))?;
            {%- endif %}

            let uniffi_return_value = {{ func.module_path }}::{{ func.callable.name_rs() }}(
                {%- for arg in func.callable.arguments %}
                {{ arg.name_rs() }},
                {%- endfor %}
            );
            {%- if let Some(return_ty) = func.callable.return_type %}
            uniffi_buf.with_cursor(|uniffi_writer| {
                {{ return_ty.write_fn_rs }}(uniffi_writer, uniffi_return_value)
            })?;
            {%- endif %}
            Ok(())
        }
     })
}
