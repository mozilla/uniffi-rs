data class {{ rec.name()|class_name_kt }} (
    {%- for field in rec.fields() %}
    var {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt -}}
    {%- match field.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_kt }}
        {%- else %}
    {%- endmatch -%}
    {% if !loop.last %}, {% endif %}
    {%- endfor %}
) {
    companion object {
        // XXX TODO: put this in a superclass maybe?
        internal fun lift(rbuf: RustBuffer.ByValue): {{ rec.name()|class_name_kt }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ rec.name()|class_name_kt }}.read(buf) }
        }

        internal fun read(buf: ByteBuffer): {{ rec.name()|class_name_kt }} {
            return {{ rec.name()|class_name_kt }}(
            {%- for field in rec.fields() %}
            {{ "buf"|read_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
            )
        }
    }

    internal fun lower(): RustBuffer.ByValue {
        return lowerIntoRustBuffer(this, {v, buf -> v.write(buf)})
    }

    internal fun write(buf: RustBufferBuilder) {
        {%- for field in rec.fields() %}
            {{ "(this.{})"|format(field.name())|write_kt("buf", field.type_()) }}
        {%- endfor %}
    }
}