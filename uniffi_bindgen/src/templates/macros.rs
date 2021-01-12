{#
// Template to receive calls into rust.
#}

{%- macro to_rs_call(func) -%}
{{ func.name() }}({% call _arg_list_rs_call(func) -%})
{%- endmacro -%}

{%- macro to_rs_call_with_prefix(prefix, func) -%}
    {{ func.name() }}(
    {{- prefix }}{% if func.arguments().len() > 0 %}, {% call _arg_list_rs_call(func) -%}{% endif -%}
)
{%- endmacro -%}

{%- macro _arg_list_rs_call(func) %}
    {%- for arg in func.arguments() %}
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

{% macro return_type_func(func) %}{% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_ffi }}{%- else -%}(){%- endmatch -%}{%- endmacro -%}

{% macro ret(func) %}{% match func.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lower_rs(return_type) }}{% else %}_retval{% endmatch %}{% endmacro %}

{% macro to_rs_constructor_call(obj, cons) %}
{% match cons.throws() %}
{% when Some with (e) %}
uniffi::deps::ffi_support::call_with_result(err, || -> Result<{{ obj.name() }}, {{e}}> {
    let _retval = {{ obj.name() }}::{% call to_rs_call(cons) %}?;
    Ok(_retval)
})
{% else %}
uniffi::deps::ffi_support::call_with_output(err, || {
    {{ obj.name() }}::{% call to_rs_call(cons) %}
})
{% endmatch %}
{% endmacro %}

{% macro to_rs_method_call(obj, meth) %}
{% match meth.throws() %}
{% when Some with (e) %}
uniffi::deps::ffi_support::call_with_result(err, || -> Result<{% call return_type_func(meth) %}, {{e}}> {
    // We're declaring the argument type as an `Option<Box<T>>` but the value is owned by the foreign code,
    // we so don't want to drop the Box. Probably it would be better to encode this as a reference type.
    let obj_box = std::mem::ManuallyDrop::new(obj.expect("Must receive a non-null object pointer"));
    // TODO: terrifically unsafe to assume we can take a mutable reference here! Needs locks etc.
    let obj = obj_box.as_ref();
    let _retval = {{ obj.name() }}::{%- call to_rs_call_with_prefix("obj", meth) -%}?;
    Ok({% call ret(meth) %})
})
{% else %}
uniffi::deps::ffi_support::call_with_output(err, || {
    // We're declaring the argument type as an `Option<Box<T>>` but the value is owned by the foreign code,
    // we so don't want to drop the Box. Probably it would be better to encode this as a reference type.
    let obj_box = std::mem::ManuallyDrop::new(obj.expect("Must receive a non-null object pointer"));
    // TODO: terrifically unsafe to assume we can take a mutable reference here! Needs locks etc.
    let obj = obj_box.as_ref();
    let _retval = {{ obj.name() }}::{%- call to_rs_call_with_prefix("obj", meth) -%};
    {% call ret(meth) %}
})
{% endmatch %}
{% endmacro %}

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
