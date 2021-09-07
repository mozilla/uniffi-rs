{% import "macros.kt" as kt %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

fun {{ func.nm() }}({%- call kt::arg_list_decl(func) -%}): {{ return_type.nm() }} {
    return {{ return_type.lift() }}({% call kt::to_ffi_call(func) %})
}

{% when None -%}

fun {{ func.nm() }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
