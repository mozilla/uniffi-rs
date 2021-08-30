{% import "macros.kt" as kt %}
{%- let rec = self.inner() %}
{% call kt::unsigned_types_annotation(self) %}
data class {{ rec.name()|class_name_kt }} (
    {%- for field in rec.fields() %}
    {%- let field_type = field.type_() %}
    var {{ field.name()|var_name_kt }}: {{ field_type|type_kt -}}
    {%- match field.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_kt(field_type) }}
        {%- else %}
    {%- endmatch -%}
    {% if !loop.last %}, {% endif %}
    {%- endfor %}
) {% if self.contains_object_references() %}: Disposable {% endif %}{
    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        {% call kt::destroy_fields(rec) %}
    }
    {% endif %}
}

{% call kt::unsigned_types_annotation(self) %}
object {{ rec.type_()|ffi_converter_name }} {
    internal fun read(buf: ByteBuffer): {{ rec.name()|class_name_kt }} {
        return {{ rec.name()|class_name_kt }}(
        {%- for field in rec.fields() %}
        {{ field.type_()|ffi_converter_name }}.read(buf){% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
        )
    }

    internal fun write(v: {{ rec.name()|class_name_kt }}, buf: RustBufferBuilder) {
        {%- for field in rec.fields() %}
            {{ field.type_()|ffi_converter_name }}.write(v.{{ field.name()|var_name_kt }}, buf)
        {% endfor %}
    }

    {% call kt::lift_and_lower_from_read_and_write(rec.type_()) %}
}
