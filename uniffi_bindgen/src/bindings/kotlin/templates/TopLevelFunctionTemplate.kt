{%- if func.is_async() %}
    {%- match func.throws_type() -%}
    {%- when Some with (throwable) %}
        @Throws({{ throwable|type_name }}::class)
    {%- else -%}
    {%- endmatch %}

    {%- call kt::async_func(func) -%}
{%- else %}
    {%- match func.throws_type() -%}
    {%- when Some with (throwable) %}
        @Throws({{ throwable|type_name }}::class)
    {%- else -%}
    {%- endmatch -%}
{% include "FunctionDocsTemplate.kt" %}
fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
    return {{ return_type|lift_fn }}({% call kt::to_ffi_call(func) %})
}

    {%- match func.return_type() -%}
    {%- when Some with (return_type) %}

        {% include "FunctionDocsTemplate.kt" %}
        fun {{ func.name()|fn_name }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_name }} {
            return {{ return_type|lift_fn }}({% call kt::to_ffi_call(func) %})
        }
    {% when None %}

        {% include "FunctionDocsTemplate.kt" %}
        fun {{ func.name()|fn_name }}({% call kt::arg_list_decl(func) %}) =
            {% call kt::to_ffi_call(func) %}

    {% endmatch %}
{%- endif %}
