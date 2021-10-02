internal fun Byte.Companion.lift(v: Byte): Byte {
    return v
}

internal fun Byte.Companion.read(buf: ByteBuffer): Byte {
    return buf.get()
}

internal fun Byte.lower(): Byte {
    return this
}

internal fun Byte.write(buf: RustBufferBuilder) {
    buf.putByte(this)
}
