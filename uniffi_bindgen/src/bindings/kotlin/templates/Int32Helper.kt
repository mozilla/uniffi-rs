internal object FfiConverterI32 {
    fun lift(v: Int): Int {
        return v
    }

    fun read(buf: ByteBuffer): Int {
        return buf.getInt()
    }

    fun lower(v: Int): Int {
        return v
    }

    fun write(v: Int, buf: RustBufferBuilder) {
        buf.putInt(v)
    }
}
