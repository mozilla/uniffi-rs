#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ jni_method_name }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    {%- for ffi_arg in callable.ffi_arguments() %}
    {{ ffi_arg.name_rs() }}: {{ ffi_arg.ty.type_rs() }},
    {%- endfor %}
)
{%- match callable.return_ffi %}
{%- when ReturnFfi::Primitive { ffi_type, .. } %} -> {{ ffi_type.type_rs() }}
{%- when ReturnFfi::Void %}
{%- endmatch %}
{
    uniffi::trace!("Calling {{ callable.name }}");
    unsafe {
        uniffi_jni::rust_call_with_env(uniffi_env, |uniffi_env| {
            {%- for arg in callable.arguments %}
            let uniffi_arg_lifted_{{ loop.index0 }} = {{ arg.ty.lift_fn_rs() }}(
                uniffi_env,
                {%- for ffi_arg in arg.ffi_args %}
                {{ ffi_arg.name_rs() }},
                {%- endfor %}
            )?;
            {%- endfor %}

            let uniffi_return = {{ callable.fully_qualified_name_rs }}(
                {%- for arg in callable.arguments %}
                uniffi_arg_lifted_{{ loop.index0 }},
                {%- endfor %}
            );
            {%- match callable.return_ffi %}
            {%- when ReturnFfi::Primitive { type_node, .. } %}
            uniffi::Result::Ok({{ type_node.lower_fn_rs() }}(uniffi_env, uniffi_return)?)
            {%- when ReturnFfi::Void %}
            uniffi::Result::Ok(())
            {% endmatch %}
        })
    }
}
