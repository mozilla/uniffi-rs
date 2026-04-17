fun readBytes(cursor: uniffi.FfiBufferCursor): ByteArray {
    val len = readULong(cursor).toInt()
    val byteArr = ByteArray(len)
    for (i in 0..<len) {
        byteArr.set(i, readByte(cursor))
    }
    return byteArr
}

fun writeBytes(cursor: uniffi.FfiBufferCursor, value: ByteArray) {
    writeULong(cursor, value.size.toULong())
    value.iterator().forEach {
        writeByte(cursor, it)
    }
}
