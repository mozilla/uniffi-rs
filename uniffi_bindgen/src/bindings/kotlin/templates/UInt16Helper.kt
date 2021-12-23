internal object FfiConverterU16 {
    fun lift(v: Short): UShort {
        return v.toUShort()
    }

    fun read(buf: ByteBuffer): UShort {
        return lift(buf.getShort())
    }

    fun lower(v: UShort): Short {
        return v.toShort()
    }

    fun write(v: UShort, buf: RustBufferBuilder) {
        buf.putShort(v.toShort())
    }
}
