{%- let type_name = cbi.self_type.type_rs %}
{%- let trait_name = "{}::{}"|format(cbi.module_path, cbi.name_rs()) %}

struct {{ cbi.impl_struct_rs() }} {
    handle: i64,
}

{% if cbi.has_async_method() %}#[uniffi::deps::async_trait::async_trait]{% endif %}
impl {{ trait_name }} for {{ cbi.impl_struct_rs() }} {
    {%- for meth in cbi.methods %}
    {%- let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}fn {{ callable.name_rs() }}(
        &self,
        {%- for a in callable.arguments %}
        {{ a.name_rs() }}: {{ a.ty.type_rs }},
        {%- endfor %}
    ) -> {{ callable.return_type_rs() }} {
        uniffi::trace!("Callback call: {{ callable.name }}");
        let mut uniffi_buf = uniffi::FfiBuffer::new();
        uniffi::trace!("{{ callable.name }}: new buffer {uniffi_buf:?}");
        let uniffi_result = {{ meth.dispatch_fn_rs }}(
            self.handle,
            &mut uniffi_buf,
            {%- for a in callable.arguments %}
            {{ a.name_rs() }},
            {%- endfor %}
        ){% if callable.is_async %}.await{% endif %};
        uniffi::trace!("{{ callable.name }}: free buffer {uniffi_buf:?}");
        uniffi_buf.free();
        match uniffi_result {
            Ok(v) => v,
            Err(e) => {
                {%- if let Some(throws_type) = callable.throws_type %}
                {%- if throws_type.has_from_unexpected_callback_error_impl %}
                Err(<{{ throws_type.type_rs }} as ::std::convert::From<uniffi::UnexpectedUniFFICallbackError>>::from(
                    uniffi::UnexpectedUniFFICallbackError {
                        reason: e.to_string(),
                    }
                ))
                {%- else %}
                panic!("Error calling UniFFI callback method: {e}")
                {%- endif %}
                {%- else %}
                panic!("Error calling UniFFI callback method: {e}")
                {%- endif %}
            }
        }
    }
    {%- endfor %}
}

{%- for meth in cbi.methods %}
{%- let callable = meth.callable %}
// Dispatch function for the {{ cbi.name }}::{{ callable.name }}
//
// Ok returns represent a regular call
// Err returns represent an unexpected error, for example failure to lift arguments
{% if callable.is_async %}async {% endif %}fn {{ meth.dispatch_fn_rs }}(
    uniffi_callback_handle: i64,
    uniffi_buf: &mut uniffi::FfiBuffer,
    {%- for a in callable.arguments %}
    {{ a.name_rs() }}: {{ a.ty.type_rs }},
    {%- endfor %}
) -> uniffi::Result<{{ callable.return_type_rs() }}> {
    {%- if !callable.arguments.is_empty() %}
    uniffi_buf.with_cursor(|uniffi_writer| {
        {%- for a in callable.arguments %}
        {{ a.ty.write_fn_rs }}(uniffi_writer, {{ a.name_rs() }})?;
        {%- endfor %}
        Ok(())
    })?;
    {%- endif %}

    {%- if !callable.is_async %}
    static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
        c"uniffi/UniffiKt",
        c"{{ meth.dispatch_fn_kt }}",
        c"(JJ)V",
    );
    // Safety:
    //
    // * uniffi_get_global_jvm() returns a valid JavaVM pointer
    // * We use the JNI API correctly
    unsafe {
        uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |uniffi_env| {
            let uniffi_result = METHOD.call_void(uniffi_env, [
                uniffi_jni::jvalue {
                    j: uniffi_callback_handle,
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
        })
    }

    {%- else %}
    static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
        c"uniffi/UniffiKt",
        c"{{ meth.dispatch_fn_kt }}",
        c"(JJJ)V",
    );
    let (uniffi_sender, uniffi_receiver) = uniffi::oneshot::channel::<i32>();
    // Safety:
    // * uniffi_get_global_jvm() returns a valid JavaVM pointer
    // * We use the JNI API correctly
    // * Closure panics won't cause `uniffi_buf` to be invalid
    // * We don't use the buffer while the Kotlin side has it
    unsafe {
        let uniffi_buf_ptr = ::std::panic::AssertUnwindSafe(uniffi_buf.as_ptr().expose_provenance());
        uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), move |uniffi_env| {
            if METHOD.call_void(uniffi_env, [
                uniffi_jni::jvalue {
                    j: uniffi_callback_handle,
                },
                uniffi_jni::jvalue {
                    j: uniffi_sender.into_raw().expose_provenance() as i64,
                },
                uniffi_jni::jvalue {
                    j: *uniffi_buf_ptr as i64
                },
            ]).is_err() {
                ((**uniffi_env).v1_2.ExceptionClear)(uniffi_env);
                eprintln!("Exception calling {{ meth.dispatch_fn_kt }}");
            }
        });
    }
    match uniffi_receiver.await {
        UNIFFI_KOTLIN_FUTURE_OK => {
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
        {%- if let Some(throws_type) = callable.throws_type %}
        UNIFFI_KOTLIN_FUTURE_ERR => {
            let uniffi_err = uniffi_buf.with_cursor(|uniffi_reader| {
                {{ throws_type.read_fn_rs }}(uniffi_reader)
            })?;
            Ok(Err(uniffi_err))
        }
        {%- endif %}
        result => uniffi::deps::anyhow::bail!("Unexpected async callback result: {result}")
    }
    {%- endif %}
}
{%- endfor %}

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

{% if !cbi.for_trait_interface %}
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

{%- endif %}
