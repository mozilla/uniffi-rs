{#
// Template to receive calls into rust.
#}

{%- macro to_rs_call(func) -%}
{{ func.name() }}({% call _arg_list_rs_call(func) -%})
{%- endmacro -%}

{%- macro _arg_list_rs_call(func) %}
    {%- for arg in func.full_arguments() %}
        {%- if arg.by_ref() %}&{% endif %}
        {{- arg.name()|lift_rs(arg.type_()) }}
        {%- if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name() }}: {{ arg.type_()|type_ffi -}}{% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
    {% if func.arguments().len() > 0 %},{% endif %} err: &mut uniffi::deps::ffi_support::ExternError,
{%- endmacro -%}

{%- macro arg_list_decl_with_prefix(prefix, meth) %}
    {{- prefix -}}
    {%- if meth.arguments().len() > 0 %}, {# whitespace #}
        {%- for arg in meth.arguments() %}
            {{- arg.name() }}: {{ arg.type_()|type_rs -}}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    {%- endif %}
{%- endmacro -%}

{% macro return_type_func(func) %}{% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi }}{%- else -%}(){%- endmatch -%}{%- endmacro -%}

{% macro ret(func) %}{% match func.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lower_rs(return_type) }}{% else %}_retval{% endmatch %}{% endmacro %}

{% macro construct(obj, cons) %}
    {{- obj.name() }}::{% call to_rs_call(cons) -%}
{% endmacro %}

{% macro to_rs_constructor_call(obj, cons) %}
{% match cons.throws() %}
{% when Some with (e) %}
    uniffi::deps::ffi_support::call_with_result(err, || -> Result<_, {{ e }}> {
        let _new = {% call construct(obj, cons) %}?;
        let _arc = std::sync::Arc::new(_new);
        Ok({{ "_arc"|lower_rs(obj.type_()) }})
    })
{% else %}
    uniffi::deps::ffi_support::call_with_output(err, || {
        let _new = {% call construct(obj, cons) %};
        let _arc = std::sync::Arc::new(_new);
        {{ "_arc"|lower_rs(obj.type_()) }}
    })
{% endmatch %}
{% endmacro %}

{% macro to_rs_method_call(obj, meth) -%}
{% match meth.throws() -%}
{% when Some with (e) -%}
uniffi::deps::ffi_support::call_with_result(err, || -> Result<{% call return_type_func(meth) %}, {{e}}> {
    let _retval =  {{ obj.name() }}::{% call to_rs_call(meth) %}?;
    Ok({% call ret(meth) %})
})
{% else %}
uniffi::deps::ffi_support::call_with_output(err, || {
    let _retval =  {{ obj.name() }}::{% call to_rs_call(meth) %};
    {% call ret(meth) %}
})
{% endmatch -%}
{% endmacro -%}

{% macro to_rs_function_call(func) %}
{% match func.throws() %}
{% when Some with (e) %}
uniffi::deps::ffi_support::call_with_result(err, || -> Result<{% call return_type_func(func) %}, {{e}}> {
    let _retval = {% call to_rs_call(func) %}?;
    Ok({% call ret(func) %})
})
{% else %}
uniffi::deps::ffi_support::call_with_output(err, || {
    let _retval = {% call to_rs_call(func) %};
    {% call ret(func) %}
})
{% endmatch %}
{% endmacro %}
