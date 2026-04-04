{%- let callable = scaffolding_function.callable %}
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ scaffolding_function.jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    uniffi_buf_handle: i64,
) {
    // Safety:
    // * uniffi_env points to a valid JNIENv
    // * We assume the Kotlin side of the FFI sent us a valid buffer handle with arguments
    //   correctly serialized.
    unsafe {
        uniffi_jni::rust_call(uniffi_env, |uniffi_env| {
            let mut uniffi_buf = uniffi::FfiBuffer::from_ptr(
                ::std::ptr::with_exposed_provenance_mut(uniffi_buf_handle as usize)
            );
 
            {%- if !callable.arguments.is_empty() || callable.has_receiver() %}
            let (
                {%- if callable.has_receiver() %}
                uniffi_self,
                {%- endif %}
                {%- for arg in callable.arguments %}
                {{ arg.name_rs() }},
                {%- endfor %}
            ) = uniffi_buf.with_cursor(|uniffi_reader| Ok((
                {%- if let Some(receiver_type) = callable.receiver_type() %}
                {{ receiver_type.read_fn_rs }}(uniffi_reader)?,
                {%- endif %}
                {%- for arg in callable.arguments %}
                {{ arg.ty.read_fn_rs }}(uniffi_reader)?,
                {%- endfor %}
            )))?;
            {%- endif %}
 
            {%- if callable.has_receiver() %}
            let uniffi_return_value = uniffi_self.{{ callable.name_rs() }}(
                {%- for arg in callable.arguments %}
                {{ arg.name_rs() }},
                {%- endfor %}
            );
            {%- else %}
            let uniffi_return_value = {{ callable.fully_qualified_name_rs }}(
                {%- for arg in callable.arguments %}
                {{ arg.name_rs() }},
                {%- endfor %}
            );
            {%- endif %}

            {%- if let Some(throws_ty) = callable.throws_type %}
            let uniffi_return_value = match uniffi_return_value {
                Ok(v) => v,
                Err(uniffi_err) => {
                    uniffi_buf.with_cursor(|uniffi_writer| {
                        {{ throws_ty.write_fn_rs }}(uniffi_writer, uniffi_err)
                    })?;
                    // Safety:
                    // `uniffi_buf` points to a valid FFI buffer
                    unsafe { {{ throws_ty.throw_error_fn_rs() }}(uniffi_env, uniffi_buf.into_ptr()); };
                    return Ok(());
                }
            };
            {%- endif %}

            {%- if let Some(return_ty) = callable.return_type %}
            uniffi_buf.with_cursor(|uniffi_writer| {
                {{ return_ty.write_fn_rs }}(uniffi_writer, uniffi_return_value)
            })?;
            {%- endif %}
            Ok(())
        })
    }
}
