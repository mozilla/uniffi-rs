class {{ name }} (internal val ptr: com.sun.jna.Pointer, internal val size: Int) {
    internal val byteBuf = ptr.getByteBuffer(0, size.toLong()).also { it.order(java.nio.ByteOrder.BIG_ENDIAN) }
    fun readString(size: Int): String {
        val byteArr = ByteArray(size)
        byteBuf.get(byteArr)
        return byteArr.toString(Charsets.UTF_8)
    }

    fun writeString(value: String) {
        byteBuf.put(value.toByteArray(Charsets.UTF_8))
    }
}
