internal fun UByte.Companion.lift(v: Byte): UByte {
    return v.toUByte()
}

internal fun UByte.Companion.read(buf: ByteBuffer): UByte {
    return UByte.lift(buf.get())
}

internal fun UByte.lower(): Byte {
    return this.toByte()
}

internal fun UByte.write(buf: RustBufferBuilder) {
    buf.putByte(this.toByte())
}