internal fun Boolean.Companion.lift(v: Byte): Boolean {
    return v.toInt() != 0
}

internal fun Boolean.Companion.read(buf: ByteBuffer): Boolean {
    return Boolean.lift(buf.get())
}

internal fun Boolean.lower(): Byte {
    return if (this) 1.toByte() else 0.toByte()
}

internal fun Boolean.write(buf: RustBufferBuilder) {
    buf.putByte(this.lower())
}