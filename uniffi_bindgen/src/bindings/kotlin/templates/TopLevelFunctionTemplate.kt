{%- match func.return_type() -%}
{%- when Some with (return_type) %}

// SAM_TODO: There is one more error I need to figure out how to also catch
// If any arguments have a long type they will need the annotation
{% if ci.contains_unsigned_type(return_type) %}@ExperimentalUnsignedTypes{% endif %}
fun {{ func.name()|fn_name_kt }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_kt }} {
    val _retval = {% call kt::to_ffi_call(func) %}
    return {{ "_retval"|lift_kt(return_type) }}
}

{% when None -%}

fun {{ func.name()|fn_name_kt }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
