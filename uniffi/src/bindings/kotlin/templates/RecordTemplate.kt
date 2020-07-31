data class {{ rec.name()|class_name_kt }} (
    {%- for field in rec.fields() %}
    val {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt }}{% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
) {
    companion object {
        // XXX TODO: put this in a superclass maybe?
        internal fun lift(rbuf: RustBuffer.ByValue): {{ rec.name()|class_name_kt }} {
            return liftFromRustBuffer(rbuf) { buf -> {{ rec.name()|class_name_kt }}.liftFrom(buf) }
        }

        internal fun liftFrom(buf: ByteBuffer): {{ rec.name()|class_name_kt }} {
            return {{ rec.name()|class_name_kt }}(
            {%- for field in rec.fields() %}
            {{ "buf"|lift_from_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
            )
        }
    }

    internal fun lower(): RustBuffer.ByValue {
        return lowerIntoRustBuffer(this, {v -> v.lowersIntoSize()}, {v, buf -> v.lowerInto(buf)})
    }

    internal fun lowersIntoSize(): Int {
        return 0 +
        {%- for field in rec.fields() %}
            {{ "(this.{})"|format(field.name())|lowers_into_size_kt(field.type_()) }}{% if loop.last %}{% else %} +{% endif %}
        {%- endfor %}
    }

    internal fun lowerInto(buf: ByteBuffer) {
        {%- for field in rec.fields() %}
            {{ "(this.{})"|format(field.name())|lower_into_kt("buf", field.type_()) }}
        {%- endfor %}
    }
}