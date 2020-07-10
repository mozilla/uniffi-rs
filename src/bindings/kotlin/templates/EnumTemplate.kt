enum class {{ e.name()|class_name_kt }} {
    {% for value in e.values() %}
    {{ value|enum_name_kt }}{% if loop.last %};{% else %},{% endif %}
    {% endfor %}

    companion object {
        internal fun lift(n: Int) =
            try { values()[n - 1] }
            catch (e: IndexOutOfBoundsException) {
                throw RuntimeException("invalid enum value, something is very wrong!!", e)
            }

        internal fun liftFrom(buf: ByteBuffer) = lift(Int.liftFrom(buf))
    }

    internal fun lower() = this.ordinal + 1

    internal fun lowersIntoSize() = 4

    internal fun lowerInto(buf: ByteBuffer) = this.lower().lowerInto(buf)
}