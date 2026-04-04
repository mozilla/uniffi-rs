{# Kotlin translation of uniffi_core/src/ffi/ffibuffer.rs #}

fun readByte(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Byte = buf.get(offset)
fun writeByte(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Byte) = buf.put(offset, value)

fun readUByte(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.UByte = buf.get(offset).toUByte()
fun writeUByte(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.UByte) = buf.put(offset, value.toByte())

fun readShort(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Short = buf.getShort(offset)
fun writeShort(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Short) = buf.putShort(offset, value)

fun readUShort(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.UShort = buf.getShort(offset).toUShort()
fun writeUShort(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.UShort) = buf.putShort(offset, value.toShort())

fun readInt(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Int = buf.getInt(offset)
fun writeInt(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Int) = buf.putInt(offset, value)

fun readUInt(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.UInt = buf.getInt(offset).toUInt()
fun writeUInt(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.UInt) = buf.putInt(offset, value.toInt())

fun readLong(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Long = buf.getLong(offset)
fun writeLong(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Long) = buf.putLong(offset, value)

fun readULong(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.ULong = buf.getLong(offset).toULong()
fun writeULong(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.ULong) = buf.putLong(offset, value.toLong())

fun readFloat(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Float = buf.getFloat(offset)
fun writeFloat(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Float) = buf.putFloat(offset, value)

fun readDouble(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Double = buf.getDouble(offset)
fun writeDouble(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Double) = buf.putDouble(offset, value)

fun readBoolean(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.Boolean {
    return buf.get(offset).toInt() == 1
}
fun writeBoolean(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.Boolean) {
    buf.put(offset, if (value) { 1.toByte() } else { 0.toByte() })
}

fun readString(buf: java.nio.ByteBuffer, offset: kotlin.Int): kotlin.String {
    return Scaffolding.ffiBufferReadString(buf, offset)
}

fun writeString(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: kotlin.String) {
    Scaffolding.ffiBufferWriteString(buf, offset, value)
}

fun readBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int): java.nio.ByteBuffer {
    return Scaffolding.ffiBufferReadBuffer(buf, offset).order(java.nio.ByteOrder.nativeOrder())
}

fun writeBuffer(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: java.nio.ByteBuffer) {
    Scaffolding.ffiBufferWriteBuffer(buf, offset, value)
}
