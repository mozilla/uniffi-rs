{#
// Template to receive calls into rust.
#}

{%- macro to_rs_call(func) -%}
{{ func.name() }}({% call _arg_list_rs_call(func) -%})
{%- endmacro -%}

{%- macro _arg_list_rs_call(func) %}
    {%- for arg in func.full_arguments() %}
        {%- if arg.by_ref() %}&{% endif %}
        {{- arg.type_()|ffi_converter }}::try_lift({{ arg.name() }}).unwrap()
        {%- if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name() }}: {{ arg.type_()|type_ffi -}},
    {%- endfor %}
    call_status: &mut uniffi::RustCallStatus
{%- endmacro -%}

{%- macro arg_list_decl_with_prefix(prefix, meth) %}
    {{- prefix -}}
    {%- if meth.arguments().len() > 0 %}, {# whitespace #}
        {%- for arg in meth.arguments() %}
            {{- arg.name() }}: {{ arg.type_()|type_rs -}}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    {%- endif %}
{%- endmacro -%}

{% macro return_signature(func) %}{% match func.ffi_func().return_type() %}{% when Some with (return_type) %} -> {% call return_type_func(func) %}{%- else -%}{%- endmatch -%}{%- endmacro -%}

{% macro return_type_func(func) %}{% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi }}{%- else -%}(){%- endmatch -%}{%- endmacro -%}

{% macro ret(func) %}{% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|ffi_converter }}::lower(_retval){% else %}_retval{% endmatch %}{% endmacro %}

{% macro construct(obj, cons) %}
    {{- obj.name() }}::{% call to_rs_call(cons) -%}
{% endmacro %}

{% macro to_rs_constructor_call(obj, cons) %}
{% match cons.throws_type() %}
{% when Some with (e) %}
    uniffi::call_with_result(call_status, || {
        let _new = {% call construct(obj, cons) %}.map_err(Into::into).map_err({{ e|ffi_converter }}::lower)?;
        let _arc = std::sync::Arc::new(_new);
        Ok({{ obj.type_()|ffi_converter }}::lower(_arc))
    })
{% else %}
    uniffi::call_with_output(call_status, || {
        let _new = {% call construct(obj, cons) %};
        let _arc = std::sync::Arc::new(_new);
        {{ obj.type_()|ffi_converter }}::lower(_arc)
    })
{% endmatch %}
{% endmacro %}

{% macro to_rs_method_call(obj, meth) -%}
{% match meth.throws_type() -%}
{% when Some with (e) -%}
uniffi::call_with_result(call_status, || {
    let _retval =  {{ obj.name() }}::{% call to_rs_call(meth) %}.map_err(Into::into).map_err({{ e|ffi_converter }}::lower)?;
    Ok({% call ret(meth) %})
})
{% else %}
uniffi::call_with_output(call_status, || {
    {% match meth.return_type() -%}
    {% when Some with (return_type) -%}
    let retval = {{ obj.name() }}::{% call to_rs_call(meth) %};
    {{ return_type|ffi_converter }}::lower(retval)
    {% else -%}
    {{ obj.name() }}::{% call to_rs_call(meth) %}
    {% endmatch -%}
})
{% endmatch -%}
{% endmacro -%}

{% macro to_rs_function_call(func) %}
{% match func.throws_type() %}
{% when Some with (e) %}
uniffi::call_with_result(call_status, || {
    let _retval = {% call to_rs_call(func) %}.map_err(Into::into).map_err({{ e|ffi_converter }}::lower)?;
    Ok({% call ret(func) %})
})
{% else %}
uniffi::call_with_output(call_status, || {
    {% match func.return_type() -%}
    {% when Some with (return_type) -%}
    {{ return_type|ffi_converter }}::lower({% call to_rs_call(func) %})
    {% else -%}
    {% call to_rs_call(func) %}
    {% endmatch -%}
})
{% endmatch %}
{% endmacro %}
