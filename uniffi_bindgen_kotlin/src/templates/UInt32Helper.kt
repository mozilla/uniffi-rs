internal fun UInt.Companion.lift(v: Int): UInt {
    return v.toUInt()
}

internal fun UInt.Companion.read(buf: ByteBuffer): UInt {
    return UInt.lift(buf.getInt())
}

internal fun UInt.lower(): Int {
    return this.toInt()
}

internal fun UInt.write(buf: RustBufferBuilder) {
    buf.putInt(this.toInt())
}