{%- let type_name = cbi.self_type.type_rs %}
{%- let trait_name = "{}::{}"|format(cbi.module_path, cbi.name_rs()) %}

struct {{ cbi.impl_struct_rs() }} {
    handle: i64,
}

impl {{ trait_name }} for {{ cbi.impl_struct_rs() }} {
    {%- for meth in cbi.methods %}
    fn {{ meth.callable.name_rs() }}(
        &self,
        {%- for a in meth.callable.arguments %}
        {{ a.name_rs() }}: {{ a.ty.type_rs }},
        {%- endfor %}
    ) -> {{ meth.callable.result.return_type_rs() }} {
        unsafe {
            let uniffi_result = {{ meth.dispatch_fn_rs }}(
                self.handle,
                {%- for a in meth.callable.arguments %}
                {{ a.name_rs() }},
                {%- endfor %}
            );
            match uniffi_result {
                Ok(v) => v,
                Err(e) => {
                    {%- if let Some(throws_type) = meth.callable.throws_type() %}
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
    }
    {%- endfor %}
}


{%- for meth in cbi.methods %}
{%- if !meth.callable.is_async %}
{# sync callback method #}
unsafe fn {{ meth.dispatch_fn_rs }}(
    uniffi_handle: ::std::primitive::i64,
    {%- for a in meth.callable.arguments %}
    {{ a.name_rs() }}: {{ a.ty.type_rs }},
    {%- endfor %}
) -> uniffi::Result<{{ meth.callable.result.return_type_rs() }}> {
    static METHOD: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
        c"uniffi/UniffiKt",
        c"{{ meth.dispatch_fn_kt }}",
        c"{{ meth.jni_signature() }}",
    );

    uniffi_jni::attach_current_thread(uniffi_get_global_jvm(), |uniffi_env| {
        {%- if meth.has_return_pointer() %}
        let mut return_value: ::std::option::Option<{{ meth.callable.result.return_type_rs() }}> = ::std::option::Option::None;
        {%- endif %}

        {%- for arg in meth.callable.arguments %}
        let uniffi_arg_lowered{{ arg.index }} = {{ arg.ty.lower_fn_rs() }}(uniffi_env, {{ arg.name_rs() }})?;
        {%- endfor %}

        let uniffi_result = METHOD.{{ meth.jni_method_call_name() }}(uniffi_env, [
            uniffi_jni::jvalue {
                j: uniffi_handle,
            },
            {%- if meth.has_return_pointer() %}
            uniffi_jni::jvalue {
                j: (&raw mut return_value).expose_provenance() as ::std::primitive::i64,
            },
            {%- endif %}
            {%- for arg in meth.callable.arguments %}
            {%- for (var, ffi_type) in arg.ty.ffi_values_rs(format!("uniffi_arg_lowered{}", arg.index)) %}
            uniffi_jni::jvalue {
                {{ ffi_type.jvalue_field() }}: {{ var }},
            },
            {%- endfor %}
            {%- endfor %}
        ]);

        match uniffi_result {
            Ok(uniffi_return) => {
                {%- if meth.has_return_pointer() %}
                let uniffi_return = match return_value {
                    Some(v) => return Ok(v),
                    {%- match meth.callable.return_ffi() %}
                    {%- when ReturnFfi::Deconstruct { .. } %}
                    None => uniffi::deps::anyhow::bail!(
                        "UniFFI internal error in {{ meth.callable.name_rs() }}: no return value set"
                    ),
                    {%- when ReturnFfi::Primitive { type_node, .. } %}
                    None => {{ type_node.lift_fn_rs() }}(uniffi_env, uniffi_return)?,
                    {%- when ReturnFfi::Void %}
                    None => ()
                    {%- endmatch %}
                };
                {%- else %}
                {%- match meth.callable.return_ffi() %}
                {%- when ReturnFfi::Primitive { type_node, .. } %}
                let uniffi_return = {{ type_node.lift_fn_rs() }}(uniffi_env, uniffi_return)?;
                {%- when ReturnFfi::Void %}
                let uniffi_return = ();
                {%- when ReturnFfi::Deconstruct { .. } %}
                {# not possible, there's always a return pointer for ReturnFfi::Deconstruct #}
                {%- endmatch %}
                {%- endif %}

                {%- if meth.callable.throws_type().is_none() %}
                Ok(uniffi_return)
                {%- else %}
                Ok(Ok(uniffi_return))
                {%- endif %}
            }
            Err(uniffi_exc) => {
                // Callback threw, handle the exception as best we can
                ((**uniffi_env).v1_2.ExceptionClear)(uniffi_env);
                Err(uniffi::deps::anyhow::anyhow!("{}", uniffi_jni::throwable_get_message(uniffi_env, uniffi_exc)))
            }
        }
    })
}
{% else %}
{# async callback method #}
async unsafe fn {{ meth.dispatch_fn_rs }}(
    uniffi_handle: ::std::primitive::i64,
    {%- for a in meth.callable.arguments %}
    {{ a.name_rs() }}: {{ a.ty.type_rs }},
    {%- endfor %}
) -> uniffi::Result<{{ meth.callable.result.return_type_rs() }}> {
    todo!("async callback methods")
}
{% endif %}
{% endfor %}

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

unsafe fn {{ cbi.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    v0: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    Ok(::std::boxed::Box::new({{ cbi.impl_struct_rs() }} {
        handle: v0
    }))
}

unsafe fn {{ cbi.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    let handle = uniffi::ffibuffer::read_i64(ptr)?;
    Ok(::std::boxed::Box::new({{ cbi.impl_struct_rs() }} {
        handle
    }))
}

// Note: no write or lower function, since passing callback interfaces from Rust to Kotlin is not allowed
