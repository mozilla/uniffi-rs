internal fun Float.Companion.lift(v: Float): Float {
    return v
}

internal fun Float.Companion.read(buf: ByteBuffer): Float {
    return buf.getFloat()
}

internal fun Float.lower(): Float {
    return this
}

internal fun Float.write(buf: RustBufferBuilder) {
    buf.putFloat(this)
}