internal fun ULong.Companion.lift(v: Long): ULong {
    return v.toULong()
}

internal fun ULong.Companion.read(buf: ByteBuffer): ULong {
    return ULong.lift(buf.getLong())
}

internal fun ULong.lower(): Long {
    return this.toLong()
}

internal fun ULong.write(buf: RustBufferBuilder) {
    buf.putLong(this.toLong())
}