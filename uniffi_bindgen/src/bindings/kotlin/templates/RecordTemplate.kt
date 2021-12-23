data class {{ type_name }} (
    {%- for field in rec.fields() %}
    var {{ field.name()|var_name }}: {{ field|type_name -}}
    {%- match field.default_value() %}
        {%- when Some with(literal) %} = {{ literal|render_literal }}
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

internal object {{ ffi_converter_name }} {
    fun lift(rbuf: RustBuffer.ByValue): {{ type_name }} {
        return liftFromRustBuffer(rbuf) { buf -> read(buf) }
    }

    fun read(buf: ByteBuffer): {{ type_name }} {
        return {{ type_name }}(
        {%- for field in rec.fields() %}
            {{ field|read_fn }}(buf),
        {%- endfor %}
        )
    }

    fun lower(value: {{ type_name }}): RustBuffer.ByValue {
        return lowerIntoRustBuffer(value, {v, buf -> write(v, buf)})
    }

    fun write(value: {{ type_name }}, buf: RustBufferBuilder) {
        {%- for field in rec.fields() %}
            {{ field|write_fn }}(value.{{ field.name()|var_name }}, buf)
        {% endfor %}
    }
}
