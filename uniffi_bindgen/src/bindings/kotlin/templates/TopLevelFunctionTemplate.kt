{% import "macros.kt" as kt %}
{%- let func = self.inner() %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

{% call kt::unsigned_types_annotation(self) %}
fun {{ func.name()|fn_name_kt }}({%- call kt::arg_list_decl(func) -%}): {{ return_type|type_kt }} {
    return {{ return_type|ffi_converter_name }}.lift({% call kt::to_ffi_call(func) %})
}

{% when None -%}

{% call kt::unsigned_types_annotation(self) %}
fun {{ func.name()|fn_name_kt }}({% call kt::arg_list_decl(func) %}) =
    {% call kt::to_ffi_call(func) %}
{% endmatch %}
