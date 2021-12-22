internal object FfiConverterShort {
    fun lift(v: Short): Short {
        return v
    }

    fun read(buf: ByteBuffer): Short {
        return buf.getShort()
    }

    fun lower(v: Short): Short {
        return v
    }

    fun write(v: Short, buf: RustBufferBuilder) {
        buf.putShort(v)
    }
}
