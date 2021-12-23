internal object FfiConverterU8 {
    fun lift(v: Byte): UByte {
        return v.toUByte()
    }

    fun read(buf: ByteBuffer): UByte {
        return lift(buf.get())
    }

    fun lower(v: UByte): Byte {
        return v.toByte()
    }

    fun write(v: UByte, buf: RustBufferBuilder) {
        buf.putByte(v.toByte())
    }
}
