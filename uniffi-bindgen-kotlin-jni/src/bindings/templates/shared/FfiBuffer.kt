
// Note: Most of this code was copied from `uniffi_core/src/ffi/ffibuffer.rs` and adapted to Kotlin
const val BASE_MINI_BUFFER_SIZE: Long = 256L

/**
 * Used to read/write from a FFIbuffer
 *
 * This tracks the current position in the buffer.
 */
class FfiBufferCursor(internal var ptr: Long) {
    internal var end = ptr + BASE_MINI_BUFFER_SIZE - 8
    internal var miniBufSize = BASE_MINI_BUFFER_SIZE

    fun minibufRemaining(): Long {
        return end - ptr
    }

    fun advanceToNextMinibuf() {
        ptr = uniffi.Scaffolding.miniBufferNext(end, miniBufSize);
        miniBufSize *= 2L
        end = ptr + miniBufSize - 8L
    }

    // Prepare for a read or write
    //
    // This ensures `this.ptr` is properly aligned and there's enough space left in the current mini
    // buffer.
    fun prepare(align: Long, size: Long) {
        // Offset needed to properly align the pointer
        val alignOffset = (-ptr).mod(align)
        if (ptr + alignOffset + size > end) {
            advanceToNextMinibuf()
        } else {
            ptr += alignOffset
        }
    }
}

fun readByte(cursor: FfiBufferCursor): Byte {
    cursor.prepare(1, 1)
    val value = Scaffolding.readByte(cursor.ptr)
    cursor.ptr += 1
    return value
}

fun readUByte(cursor: FfiBufferCursor): UByte {
    return readByte(cursor).toUByte()
}

fun readShort(cursor: FfiBufferCursor): Short {
    cursor.prepare(2, 2)
    val value = Scaffolding.readShort(cursor.ptr)
    cursor.ptr += 2
    return value
}

fun readUShort(cursor: FfiBufferCursor): UShort {
    return readShort(cursor).toUShort()
}

fun readInt(cursor: FfiBufferCursor): Int {
    cursor.prepare(4, 4)
    val value = Scaffolding.readInt(cursor.ptr)
    cursor.ptr += 4
    return value
}

fun readUInt(cursor: FfiBufferCursor): UInt {
    return readInt(cursor).toUInt()
}

fun readLong(cursor: FfiBufferCursor): Long {
    cursor.prepare(8, 8)
    val value = Scaffolding.readLong(cursor.ptr)
    cursor.ptr += 8
    return value
}

fun readULong(cursor: FfiBufferCursor): ULong {
    return readLong(cursor).toULong()
}

fun readFloat(cursor: FfiBufferCursor): Float {
    cursor.prepare(4, 4)
    val value = Scaffolding.readFloat(cursor.ptr)
    cursor.ptr += 4
    return value
}

fun readDouble(cursor: FfiBufferCursor): Double {
    cursor.prepare(8, 8)
    val value = Scaffolding.readDouble(cursor.ptr)
    cursor.ptr += 8
    return value
}

fun readBool(cursor: FfiBufferCursor): Boolean {
    return readByte(cursor) == 1.toByte()
}

fun readString(cursor: FfiBufferCursor): String {
    // Strings are stored as 3 64-bit values
    cursor.prepare(8, 24)
    val value = Scaffolding.readString(cursor.ptr)
    cursor.ptr += 24
    return value
}

fun writeByte(cursor: FfiBufferCursor, value: Byte) {
    cursor.prepare(1, 1)
    Scaffolding.writeByte(cursor.ptr, value)
    cursor.ptr += 1
}

fun writeUByte(cursor: FfiBufferCursor, value: UByte) {
    writeByte(cursor, value.toByte())
}

fun writeShort(cursor: FfiBufferCursor, value: Short) {
    cursor.prepare(2, 2)
    Scaffolding.writeShort(cursor.ptr, value)
    cursor.ptr += 2
}

fun writeUShort(cursor: FfiBufferCursor, value: UShort) {
    writeShort(cursor, value.toShort())
}

fun writeInt(cursor: FfiBufferCursor, value: Int) {
    cursor.prepare(4, 4)
    Scaffolding.writeInt(cursor.ptr, value)
    cursor.ptr += 4
}

fun writeUInt(cursor: FfiBufferCursor, value: UInt) {
    writeInt(cursor, value.toInt())
}

fun writeLong(cursor: FfiBufferCursor, value: Long) {
    cursor.prepare(8, 8)
    Scaffolding.writeLong(cursor.ptr, value)
    cursor.ptr += 8
}

fun writeULong(cursor: FfiBufferCursor, value: ULong) {
    writeLong(cursor, value.toLong())
}

fun writeFloat(cursor: FfiBufferCursor, value: Float) {
    cursor.prepare(4, 4)
    Scaffolding.writeFloat(cursor.ptr, value)
    cursor.ptr += 4
}

fun writeDouble(cursor: FfiBufferCursor, value: Double) {
    cursor.prepare(8, 8)
    Scaffolding.writeDouble(cursor.ptr, value)
    cursor.ptr += 8
}

fun writeBool(cursor: FfiBufferCursor, value: Boolean) {
    writeByte(cursor, if (value) { 1.toByte() } else { 0.toByte() })
}

fun writeString(cursor: FfiBufferCursor, value: String) {
    // Strings are stored as 3 64-bit values
    cursor.prepare(8, 24)
    Scaffolding.writeString(cursor.ptr, value)
    cursor.ptr += 24
}
