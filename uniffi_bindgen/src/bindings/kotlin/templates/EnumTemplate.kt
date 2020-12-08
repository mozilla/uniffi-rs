enum class {{ e.name()|class_name_kt }} {
    {% for variant in e.variants() %}
    {{ variant|enum_variant_kt }}{% if loop.last %};{% else %},{% endif %}
    {% endfor %}

    companion object {
        internal fun lift(n: Int) =
            try { values()[n - 1] }
            catch (e: IndexOutOfBoundsException) {
                throw RuntimeException("invalid enum value, something is very wrong!!", e)
            }

        internal fun read(buf: ByteBuffer) = lift(buf.getInt())
    }

    internal fun lower() = this.ordinal + 1

    internal fun write(buf: RustBufferBuilder) = buf.putInt(this.lower())
}
