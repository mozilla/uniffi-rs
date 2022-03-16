{% import "macros.kt" as kt %}
{%- let func = self.inner() %}
{%- match func.throws() -%}
{%- when Some with (throwable) %}
@Throws({{ throwable|exception_name }}::class)
{%- else -%}
{%- endmatch %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

{% if internalize %}internal {% endif %}fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    return {{ return_type|lift_fn }}({% call kt::to_ffi_call(func) %})
}

{% when None -%}

{% if internalize %}internal {% endif %}fun {{ func.name()|fn_name }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
