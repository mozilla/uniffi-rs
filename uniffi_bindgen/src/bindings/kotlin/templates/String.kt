object FFIConverterString: FFIConverter<String, RustBuffer.ByValue> {
    override fun lift(v: RustBuffer.ByValue): String {
        try {
            val byteArr = ByteArray(v.len)
            v.asByteBuffer()!!.get(byteArr)
            return byteArr.toString(Charsets.UTF_8)
        } finally {
            RustBuffer.free(v)
        }
    }

    override fun read(buf: ByteBuffer): String {
        val len = buf.getInt()
        val byteArr = ByteArray(len)
        buf.get(byteArr)
        return byteArr.toString(Charsets.UTF_8)
    }

    override fun lower(v: String): RustBuffer.ByValue {
        val byteArr = v.toByteArray(Charsets.UTF_8)
        // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
        // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
        val rbuf = RustBuffer.alloc(byteArr.size)
        rbuf.asByteBuffer()!!.put(byteArr)
        return rbuf
    }

    override fun write(v: String, buf: RustBufferBuilder) {
        val byteArr = v.toByteArray(Charsets.UTF_8)
        buf.putInt(byteArr.size)
        buf.put(byteArr)
    }
}
