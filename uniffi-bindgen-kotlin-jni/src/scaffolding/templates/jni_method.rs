{%- if !callable.is_async %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    {%- for ffi_arg in callable.ffi_arguments_including_receiver() %}
    {{ ffi_arg.name_rs() }}: {{ ffi_arg.ty.type_rs() }},
    {%- endfor %}
)
{%- match callable.return_ffi() %}
{%- when ReturnFfi::Primitive { ffi_type, .. } %} -> {{ ffi_type.type_rs() }}
{%- when ReturnFfi::Deconstruct { .. } %} -> uniffi_jni::jobject
{%- when ReturnFfi::Void %}
{%- endmatch %}
{
    uniffi::trace!("Calling {{ callable.name }}");
    unsafe {
        uniffi_jni::rust_call_with_env(uniffi_env, |uniffi_env| {
            {%- for arg in callable.arguments_including_receiver() %}
            let uniffi_arg_lifted_{{ loop.index0 }} = {{ arg.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_arg in arg.ffi_args() %}
                {{ ffi_arg.name_rs() }},
                {%- endfor %}
            )?;
            {%- endfor %}

            {%- if !callable.has_receiver() %}
            let uniffi_return = {{ callable.fully_qualified_name_rs }}(
                {%- for arg in callable.arguments %}
                {{ arg.pass_to_rust_fn(format!("uniffi_arg_lifted_{}", arg.index)) }},
                {%- endfor %}
            );
            {%- else %}
            let uniffi_return = uniffi_arg_lifted_0.{{ callable.name_rs() }}(
                {%- for arg in callable.arguments %}
                {{ arg.pass_to_rust_fn(format!("uniffi_arg_lifted_{}", arg.index)) }},
                {%- endfor %}
            );
            {%- endif %}

            {%- if let Some(throws_type) = callable.throws_type() %}
            let uniffi_return = match uniffi_return {
                uniffi::Result::Ok(v) => v,
                uniffi::Result::Err(uniffi_err) => {
                    let uniffi_err_deconstructed = {{ throws_type.lower_fn_rs() }}(uniffi_env, uniffi_err)?;
                    let uniffi_err_obj = {{ throws_type.lift_kt_from_rust_var() }}.call_object(
                        uniffi_env,
                        [
                            {%- for (var, ffi_type) in throws_type.ffi_values_rs("uniffi_err_deconstructed") %}
                            uniffi_jni::jvalue {
                                {{ ffi_type.jvalue_field() }}: {{ var }},
                            },
                            {%- endfor %}
                        ]
                    ).to_anyhow_result(uniffi_env, "{{ throws_type.lift_fn_kt() }}")?;
                    if ((**uniffi_env).v1_4.Throw)(uniffi_env, uniffi_err_obj) != 0 {
                        uniffi::deps::anyhow::bail!("Failed to throw exception for {{ jni_method_name }}");
                    }
                    return uniffi::Result::Ok(::std::default::Default::default());
                }
            };
            {%- endif %}

            {%- match callable.return_ffi() %}
            {%- when ReturnFfi::Primitive { type_node, .. } %}
            uniffi::Result::Ok({{ type_node.lower_fn_rs() }}(uniffi_env, uniffi_return)?)

            {%- when ReturnFfi::Deconstruct { type_node, ffi_types } %}
            let uniffi_return_deconstructed = {{ type_node.lower_fn_rs() }}(uniffi_env, uniffi_return)?;
            let uniffi_return_obj = {{ type_node.lift_kt_from_rust_var() }}.call_object(
                uniffi_env,
                [
                    {%- for ffi_type in ffi_types %}
                    uniffi_jni::jvalue {
                        {{ ffi_type.jvalue_field() }}: uniffi_return_deconstructed.{{ loop.index0 }},
                    },
                    {%- endfor %}
                ]
            ).to_anyhow_result(uniffi_env, "{{ type_node.lift_fn_kt() }}")?;
            uniffi::Result::Ok(uniffi_return_obj)
            {%- when ReturnFfi::Void %}
            uniffi::Result::Ok(())
            {% endmatch %}
        })
    }
}

{% else %}
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    {%- for ffi_arg in callable.ffi_arguments_including_receiver() %}
    {{ ffi_arg.name_rs() }}: {{ ffi_arg.ty.type_rs() }},
    {%- endfor %}
) -> i64 {
    uniffi::trace!("Calling {{ callable.name }}");
    unsafe {
        uniffi_jni::rust_call_with_env(uniffi_env, |uniffi_env| {
            {%- for arg in callable.arguments_including_receiver() %}
            let uniffi_arg_lifted_{{ arg.index }} = {{ arg.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_arg in arg.ffi_args() %}
                {{ ffi_arg.name_rs() }},
                {%- endfor %}
            )?;
            {%- endfor %}

            let uniffi_future = async move {
                {%- if !callable.has_receiver() %}
                {{ callable.fully_qualified_name_rs }}(
                    {%- for arg in callable.arguments %}
                    uniffi_arg_lifted_{{ arg.index }},
                    {%- endfor %}
                ).await
                {%- else %}
                uniffi_arg_lifted_0.{{ callable.name_rs() }}(
                    {%- for arg in callable.arguments %}
                    uniffi_arg_lifted_{{ arg.index }},
                    {%- endfor %}
                ).await
                {%- endif %}
            };
            Ok(UniffiRustFuture::<{{ callable.result.return_type_rs() }}>::new(uniffi_future).into_handle())
        })
    }
}
{% endif %}
