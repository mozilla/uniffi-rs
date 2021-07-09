{%- match func.return_type() -%}
{%- when Some with (return_type) %}

{% if ci.contains_unsigned_type(return_type) || ci.args_contains_unsigned(func.arguments())  %}@ExperimentalUnsignedTypes{% endif %}
fun {{ func.name()|fn_name_kt }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_kt }} {
    val _retval = {% call kt::to_ffi_call(func) %}
    return {{ "_retval"|lift_kt(return_type) }}
}

{% when None -%}

fun {{ func.name()|fn_name_kt }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
