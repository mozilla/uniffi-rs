{% import "macros.kt" as kt %}

{%- let class_name=name|class_name_kt %}
{%- let wrapped_ffi_type=wrapped_type|type_ffi_for_type %}
{%- let wrapped_ffi_converter=wrapped_type|ffi_converter_name %}
data class {{ class_name }} (val value: {{ wrapped_type|type_kt }})

{% call kt::unsigned_types_annotation(self) %}
object {{ type_|ffi_converter_name }}: FFIWrapper<{{ class_name }}, {{ wrapped_type|type_kt }}, {{ wrapped_ffi_type }}>({{ wrapped_ffi_converter }}) {
    override fun wrap(v: {{ wrapped_type|type_kt }}) = {{ class_name}}(v)
    override fun unwrap(v: {{ class_name }}) = v.value
}
