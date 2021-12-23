internal object FfiConverterI64 {
    fun lift(v: Long): Long {
        return v
    }

    fun read(buf: ByteBuffer): Long {
        return buf.getLong()
    }

    fun lower(v: Long): Long {
        return v
    }

    fun write(v: Long, buf: RustBufferBuilder) {
        buf.putLong(v)
    }
}
