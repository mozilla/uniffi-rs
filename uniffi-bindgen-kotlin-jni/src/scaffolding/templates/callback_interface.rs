{%- let type_name = cbi.self_type.type_rs %}
{%- let trait_name = "{}::{}"|format(cbi.module_path, cbi.name_rs()) %}

struct {{ cbi.impl_struct_rs() }} {
    handle: i64,
}

impl {{ trait_name }} for {{ cbi.impl_struct_rs() }} {
    {%- for meth in cbi.methods %}
    {%- let callable = meth.callable %}
    fn {{ callable.name_rs() }}(
        &self,
        {%- for a in callable.arguments %}
        {{ a.name_rs() }}: {{ a.ty.type_rs }},
        {%- endfor %}
    ) -> {{ callable.return_type_rs() }} {
        static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
            c"uniffi/UniffiKt",
            c"{{ meth.dispatch_fn_kt }}",
            c"(JJ)V",
        );

        let mut uniffi_buf = uniffi::FfiBuffer::new();
        let uniffi_result = uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |uniffi_env| {
            {%- if !callable.arguments.is_empty() %}
            uniffi_buf.with_cursor(|uniffi_writer| {
                {%- for a in callable.arguments %}
                {{ a.ty.write_fn_rs }}(uniffi_writer, {{ a.name_rs() }})?;
                {%- endfor %}
                Ok(())
            })?;
            {%- endif %}

            // Safety:
            // We use the JNI API correctly
            unsafe {
                let uniffi_result = METHOD.call_void(uniffi_env, [
                    uniffi_jni::jvalue {
                        j: self.handle,
                    },
                    uniffi_jni::jvalue {
                        j: uniffi_buf.as_ptr().expose_provenance() as i64,
                    },
                ]);
                match uniffi_result {
                    Ok(()) => {
                        // Callback returned normally, read the return value
                        {%- if let Some(return_ty) = callable.return_type %}
                        let uniffi_return = uniffi_buf.with_cursor(|uniffi_reader| {
                            {{ return_ty.read_fn_rs }}(uniffi_reader)
                        })?;
                        {%- else %}
                        let uniffi_return = ();
                        {%- endif %}
                        {%- if callable.throws_type.is_some() %}
                        Ok(Ok(uniffi_return))
                        {%- else %}
                        Ok(uniffi_return)
                        {%- endif %}
                    }
                    Err(uniffi_exc) => {
                        // Callback threw, handle the exception as best we can
                        ((**uniffi_env).v1_2.ExceptionClear)(uniffi_env);
                        {%- if let Some(throws_type) = callable.throws_type %}
                        if uniffi_jni::is_callback_exception(uniffi_env, uniffi_exc) {
                            // Handle the case the callback throwing a `uniffi.CallbackException` error
                            //
                            // In this case we need to read the E side of the Result from the FFI buffer
                            let uniffi_err = uniffi_buf.with_cursor(|uniffi_reader| {
                                {{ throws_type.read_fn_rs }}(uniffi_reader)
                            })?;
                            return Ok(Err(uniffi_err));
                        }
                        {%- endif %}
                        Err(uniffi::deps::anyhow::anyhow!("{}", uniffi_jni::throwable_get_message(uniffi_env, uniffi_exc)))
                    }
                }
            }
        });
        uniffi_buf.free();
        match uniffi_result {
            Ok(v) => v,
            // TODO: try to map unexpected callback errors to Rust errors
            Err(e) => panic!("Error calling UniFFI callback method: {e}"),
        }
    }
    {%- endfor %}
}

impl Drop for {{ cbi.impl_struct_rs() }} {
    fn drop(&mut self) {
        static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
            c"uniffi/UniffiKt",
            c"{{ cbi.free_fn_kt() }}",
            c"(J)V",
        );

        // Safety:
        //
        // * uniffi_get_global_jvm() returns a valid JavaVM pointer
        // * The arguments match the method signature
        unsafe {
            uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |env| {
                if METHOD.call_void(env, [
                    uniffi_jni::jvalue {
                        j: self.handle,
                    }
                ]).is_err() {
                    ((**env).v1_2.ExceptionClear)(env);
                    eprintln!("Exception calling {{ cbi.free_fn_kt() }}");
                }
            });
        }
    }
}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ cbi.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let handle = cursor.read_i64()?;
    Ok(::std::boxed::Box::new({{ cbi.impl_struct_rs() }} {
        handle
    }))
}

// Note: no write function, since passing callback interfaces from Rust to Kotlin is not allowed
