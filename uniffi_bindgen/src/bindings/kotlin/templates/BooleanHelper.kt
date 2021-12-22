internal object FfiConverterBoolean {
    fun lift(v: Byte): Boolean {
        return v.toInt() != 0
    }

    fun read(buf: ByteBuffer): Boolean {
        return lift(buf.get())
    }

    fun lower(v: Boolean): Byte {
        return if (v) 1.toByte() else 0.toByte()
    }

    fun write(v: Boolean, buf: RustBufferBuilder) {
        buf.putByte(lower(v))
    }
}
