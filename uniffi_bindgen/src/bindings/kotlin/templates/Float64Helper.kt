internal object FfiConverterF64 {
    fun lift(v: Double): Double {
        return v
    }

    fun read(buf: ByteBuffer): Double {
        return buf.getDouble()
    }

    fun lower(v: Double): Double {
        return v
    }

    fun write(v: Double, buf: RustBufferBuilder) {
        buf.putDouble(v)
    }
}
