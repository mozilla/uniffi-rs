internal object FfiConverterU64 {
    fun lift(v: Long): ULong {
        return v.toULong()
    }

    fun read(buf: ByteBuffer): ULong {
        return lift(buf.getLong())
    }

    fun lower(v: ULong): Long {
        return v.toLong()
    }

    fun write(v: ULong, buf: RustBufferBuilder) {
        buf.putLong(v.toLong())
    }
}
