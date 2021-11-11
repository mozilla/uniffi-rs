{% import "macros.kt" as kt %}
{%- let rec = self.inner() %}
data class {{ rec|type_name }} (
    {%- for field in rec.fields() %}
    var {{ field.name()|var_name }}: {{ field|type_name -}}
    {%- match field.default_value() %}
        {%- when Some with(literal) %} = {{ literal|render_literal(field) }}
        {%- else %}
    {%- endmatch -%}
    {% if !loop.last %}, {% endif %}
    {%- endfor %}
) {% if self.contains_object_references() %}: Disposable {% endif %}{
    companion object {
        internal fun lift(rbuf: RustBuffer.ByValue): {{ rec|type_name }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ rec|type_name }}.read(buf) }
        }

        internal fun read(buf: ByteBuffer): {{ rec|type_name }} {
            return {{ rec|type_name }}(
            {%- for field in rec.fields() %}
            {{ "buf"|read_var(field) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
            )
        }
    }

    internal fun lower(): RustBuffer.ByValue {
        return lowerIntoRustBuffer(this, {v, buf -> v.write(buf)})
    }

    internal fun write(buf: RustBufferBuilder) {
        {%- for field in rec.fields() %}
            {{ "this.{}"|format(field.name())|write_var("buf", field) }}
        {% endfor %}
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        {% call kt::destroy_fields(rec) %}
    }
    {% endif %}
}
