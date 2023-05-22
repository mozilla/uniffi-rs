@OptIn(kotlin.ExperimentalUnsignedTypes::class)
public object FfiConverterUByteArray: FfiConverterRustBuffer<UByteArray> {
    override fun read(buf: ByteBuffer): UByteArray {
        val len = buf.getInt()
        val byteArr = ByteArray(len)
        buf.get(byteArr)
        return byteArr.toUByteArray()
    }
    override fun allocationSize(value: UByteArray): Int {
        return 4 + value.size
    }
    override fun write(value: UByteArray, buf: ByteBuffer) {
        buf.putInt(value.size)
        buf.put(value.toByteArray())
    }
}
