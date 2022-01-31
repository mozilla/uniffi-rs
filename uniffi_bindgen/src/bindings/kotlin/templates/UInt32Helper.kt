internal object FfiConverterUInt {
    fun lift(v: Int): UInt {
        return v.toUInt()
    }

    fun read(buf: ByteBuffer): UInt {
        return lift(buf.getInt())
    }

    fun lower(v: UInt): Int {
        return v.toInt()
    }

    fun write(v: UInt, buf: RustBufferBuilder) {
        buf.putInt(v.toInt())
    }
}
