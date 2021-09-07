{% import "macros.kt" as kt %}
data class {{ rec.nm() }} (
    {%- for field in rec.fields() %}
    var {{ field.nm() }}: {{ field.type_().nm() -}}
    {%- match field.default_value() %}
        {%- when Some with(literal) %} = {{ field.type_().literal(literal) }}
        {%- else %}
    {%- endmatch -%}
    {% if !loop.last %}, {% endif %}
    {%- endfor %}
) {% if contains_object_references %}: Disposable {% endif %}{

    {% if contains_object_references %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        {% call kt::destroy_fields(rec) %}
    }
    {% endif %}
}

object {{ rec.ffi_converter_name() }}: FFIConverterRustBuffer<{{ rec.nm() }}> {
    override fun read(buf: ByteBuffer): {{ rec.nm() }} {
        return {{ rec.nm() }}(
        {%- for field in rec.fields() %}
        {{ field.type_().read() }}(buf){% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
        )
    }

    override fun write(v: {{ rec.nm() }}, buf: RustBufferBuilder) {
        {%- for field in rec.fields() %}
            {{ field.type_().write() }}(v.{{ field.nm() }}, buf)
        {% endfor %}
    }
}
