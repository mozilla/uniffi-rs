internal object FfiConverterFloat {
    fun lift(v: Float): Float {
        return v
    }

    fun read(buf: ByteBuffer): Float {
        return buf.getFloat()
    }

    fun lower(v: Float): Float {
        return v
    }

    fun write(v: Float, buf: RustBufferBuilder) {
        buf.putFloat(v)
    }
}
