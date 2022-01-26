internal fun UShort.Companion.lift(v: Short): UShort {
    return v.toUShort()
}

internal fun UShort.Companion.read(buf: ByteBuffer): UShort {
    return UShort.lift(buf.getShort())
}

internal fun UShort.lower(): Short {
    return this.toShort()
}

internal fun UShort.write(buf: RustBufferBuilder) {
    buf.putShort(this.toShort())
}