internal fun Short.Companion.lift(v: Short): Short {
    return v
}

internal fun Short.Companion.read(buf: ByteBuffer): Short {
    return buf.getShort()
}

internal fun Short.lower(): Short {
    return this
}

internal fun Short.write(buf: RustBufferBuilder) {
    buf.putShort(this)
}