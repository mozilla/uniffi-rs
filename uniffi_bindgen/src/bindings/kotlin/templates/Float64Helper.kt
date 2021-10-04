internal fun Double.Companion.lift(v: Double): Double {
    return v
}

internal fun Double.Companion.read(buf: ByteBuffer): Double {
    val v = buf.getDouble()
    return v
}

internal fun Double.lower(): Double {
    return this
}

internal fun Double.write(buf: RustBufferBuilder) {
    buf.putDouble(this)
}
