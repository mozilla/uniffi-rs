internal fun Long.Companion.lift(v: Long): Long {
    return v
}

internal fun Long.Companion.read(buf: ByteBuffer): Long {
    return buf.getLong()
}

internal fun Long.lower(): Long {
    return this
}

internal fun Long.write(buf: RustBufferBuilder) {
    buf.putLong(this)
}