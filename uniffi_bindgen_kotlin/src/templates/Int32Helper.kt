internal fun Int.Companion.lift(v: Int): Int {
    return v
}

internal fun Int.Companion.read(buf: ByteBuffer): Int {
    return buf.getInt()
}

internal fun Int.lower(): Int {
    return this
}

internal fun Int.write(buf: RustBufferBuilder) {
    buf.putInt(this)
}