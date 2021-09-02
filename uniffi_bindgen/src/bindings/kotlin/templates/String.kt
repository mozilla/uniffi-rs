internal fun String.Companion.lift(rbuf: RustBuffer.ByValue): String {
    try {
        val byteArr = ByteArray(rbuf.len)
        rbuf.asByteBuffer()!!.get(byteArr)
        return byteArr.toString(Charsets.UTF_8)
    } finally {
        RustBuffer.free(rbuf)
    }
}

internal fun String.Companion.read(buf: ByteBuffer): String {
    val len = buf.getInt()
    val byteArr = ByteArray(len)
    buf.get(byteArr)
    return byteArr.toString(Charsets.UTF_8)
}

internal fun String.lower(): RustBuffer.ByValue {
    val byteArr = this.toByteArray(Charsets.UTF_8)
    // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
    // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
    val rbuf = RustBuffer.alloc(byteArr.size)
    rbuf.asByteBuffer()!!.put(byteArr)
    return rbuf
}

internal fun String.write(buf: RustBufferBuilder) {
    val byteArr = this.toByteArray(Charsets.UTF_8)
    buf.putInt(byteArr.size)
    buf.put(byteArr)
}
