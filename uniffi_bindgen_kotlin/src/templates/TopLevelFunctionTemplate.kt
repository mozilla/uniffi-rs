{% import "macros.kt" as kt %}
{%- let func = self.inner() %}
{%- match func.throws() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|exception_name }}::class)
{%- else -%}
{%- endmatch %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    val _retval = {% call kt::to_ffi_call(func) %}
    return {{ "_retval"|lift_var(return_type) }}
}

{% when None -%}

fun {{ func.name()|fn_name }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
