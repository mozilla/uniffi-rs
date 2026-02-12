{%- let rec = ci.get_record_definition(name).unwrap() %}

{%- if rec.has_fields() %}
{%- call kt::docstring(rec, 0) %}
data class {{ type_name }} (
    {%- for field in rec.fields() %}
    {%- call kt::docstring(field, 4) %}
    {% if config.generate_immutable_records() %}val{% else %}var{% endif %} {{ field.name()|var_name }}: {{ field|type_name(ci) -}}
    {%- if let Some(default) = field.default_value() %} = {{ default|render_default(field, ci) }} {% endif %}
    {% if !loop.last %}, {% endif %}
    {%- endfor %}
    {%- let uniffi_trait_methods = rec.uniffi_trait_methods() %}
)
{{- self::record_interface_list(ci, rec, type_name, contains_object_references)? }}
{
    {% for meth in rec.methods() -%}
    {%- call kt::func_decl("", meth, 4) %}
    {% endfor %}

    {% call kt::uniffi_trait_impls(uniffi_trait_methods) %}
    {% if contains_object_references %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        {% call kt::destroy_fields(rec) %}
    }
    {% endif %}
    companion object
}
{%- else -%}
{%- call kt::docstring(rec, 0) %}
class {{ type_name }} {
    override fun equals(other: Any?): Boolean {
        return other is {{ type_name }}
    }

    override fun hashCode(): Int {
        return javaClass.hashCode()
    }

    companion object
}
{%- endif %}

/**
 * @suppress
 */
public object {{ rec|ffi_converter_name }}: FfiConverterRustBuffer<{{ type_name }}> {
    override fun read(buf: ByteBuffer): {{ type_name }} {
        {%- if rec.has_fields() %}
        return {{ type_name }}(
        {%- for field in rec.fields() %}
            {{ field|read_fn }}(buf),
        {%- endfor %}
        )
        {%- else %}
        return {{ type_name }}()
        {%- endif %}
    }

    override fun allocationSize(value: {{ type_name }}) = {%- if rec.has_fields() %} (
        {%- for field in rec.fields() %}
            {{ field|allocation_size_fn }}(value.{{ field.name()|var_name }}){% if !loop.last %} +{% endif %}
        {%- endfor %}
    ) {%- else %} 0UL {%- endif %}

    override fun write(value: {{ type_name }}, buf: ByteBuffer) {
        {%- for field in rec.fields() %}
            {{ field|write_fn }}(value.{{ field.name()|var_name }}, buf)
        {%- endfor %}
    }
}
