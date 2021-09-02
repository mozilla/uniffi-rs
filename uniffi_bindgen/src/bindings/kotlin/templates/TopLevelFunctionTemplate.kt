{% import "macros.kt" as kt %}
{%- let func = self.inner() %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

fun {{ func.nm() }}({%- call kt::arg_list_decl(func) -%}): {{ return_type.nm() }} {
    val _retval = {% call kt::to_ffi_call(func) %}
    return {{ return_type.lift("_retval") }}
}

{% when None -%}

fun {{ func.nm() }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
