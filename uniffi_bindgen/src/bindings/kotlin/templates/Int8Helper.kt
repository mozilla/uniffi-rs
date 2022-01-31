internal object FfiConverterByte {
    fun lift(v: Byte): Byte {
        return v
    }

    fun read(buf: ByteBuffer): Byte {
        return buf.get()
    }

    fun lower(v: Byte): Byte {
        return v
    }

    fun write(v: Byte, buf: RustBufferBuilder) {
        buf.putByte(v)
    }
}
