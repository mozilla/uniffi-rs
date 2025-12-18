// Type that can be passed across the FFI by serializing it into a buffer and passing the pointer.
//
// This is how the pointer FFI passes values alongside the `FfiConverter` interface.
// `FfiConverter` is responsible for converting between FFI values and Kotlin types and
// `UniffiFfiSerializer` is responsible for passing the FFI values across the FFI.
//
// There's some overlap between the two types and the overall FFI can certainly be improved.
// We intend to do that as part of the FFI v1 work. The short-term goal is to pass things across the
// FFI without running into JNA crashes, but leave the rest of the FFI the same
private interface UniffiFfiSerializer<KotlinType> {
    // Size in bytes needed to store this value
    //
    // Must be a multiple of 8
    fun size(): Long

    // Read an FFI value from a buffer
    fun read(buf: UniffiBufferCursor): KotlinType

    // Write an FFI value to a buffer
    fun write(buf: UniffiBufferCursor, value: KotlinType)

}

// Helper class for reading/writing to a buffer
//
// Maintains a pointer and a position in the stream.
// Each read/write advances the pointer 8 bytes, since all values are aligned by that amount in the
// pointer FFI.
private class UniffiBufferCursor(val pointer: Pointer, var offset: Long = 0) {
    fun readByte(): Byte = pointer.getByte(offset).also { offset += 8 }
    fun writeByte(value: Byte) = pointer.setByte(offset, value).also { offset += 8 }
    fun readShort(): Short = pointer.getShort(offset).also { offset += 8 }
    fun writeShort(value: Short) = pointer.setShort(offset, value).also { offset += 8 }
    fun readInt(): Int = pointer.getInt(offset).also { offset += 8 }
    fun writeInt(value: Int) = pointer.setInt(offset, value).also { offset += 8 }
    fun readLong(): Long = pointer.getLong(offset).also { offset += 8 }
    fun writeLong(value: Long) = pointer.setLong(offset, value).also { offset += 8 }
    fun readFloat(): Float = pointer.getFloat(offset).also { offset += 8 }
    fun writeFloat(value: Float) = pointer.setFloat(offset, value).also { offset += 8 }
    fun readDouble(): Double = pointer.getDouble(offset).also { offset += 8 }
    fun writeDouble(value: Double) = pointer.setDouble(offset, value).also { offset += 8 }
    fun advance(amount: Long) {
        this.offset += amount;
    }
}

private object UniffiFfiSerializerByte: UniffiFfiSerializer<Byte> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readByte()
    override fun write(buf: UniffiBufferCursor, value: Byte) = buf.writeByte(value)
}

private object UniffiFfiSerializerShort: UniffiFfiSerializer<Short> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readShort()
    override fun write(buf: UniffiBufferCursor, value: Short) = buf.writeShort(value)
}

private object UniffiFfiSerializerInt: UniffiFfiSerializer<Int> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readInt()
    override fun write(buf: UniffiBufferCursor, value: Int) = buf.writeInt(value)
}

private object UniffiFfiSerializerLong: UniffiFfiSerializer<Long> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readLong()
    override fun write(buf: UniffiBufferCursor, value: Long) = buf.writeLong(value)
}

private object UniffiFfiSerializerFloat: UniffiFfiSerializer<Float> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readFloat()
    override fun write(buf: UniffiBufferCursor, value: Float) = buf.writeFloat(value)
}

private object UniffiFfiSerializerDouble: UniffiFfiSerializer<Double> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readDouble()
    override fun write(buf: UniffiBufferCursor, value: Double) = buf.writeDouble(value)
}

private object UniffiFfiSerializerPointer: UniffiFfiSerializer<Pointer> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor): Pointer {
        return Pointer(buf.readLong())
    }
    override fun write(buf: UniffiBufferCursor, value: Pointer) {
        buf.writeLong(Pointer.nativeValue(value))
    }
}

private object UniffiFfiSerializerHandle: UniffiFfiSerializer<Long> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor) = buf.readLong()
    override fun write(buf: UniffiBufferCursor, value: Long) = buf.writeLong(value)
}

private object UniffiFfiSerializerCallback: UniffiFfiSerializer<UniffiCallbackFunction> {
    override fun size(): Long = 8
    override fun read(buf: UniffiBufferCursor): UniffiCallbackFunction {
        return CallbackReference.getCallback(UniffiCallbackFunction::class.java, Pointer(buf.readLong())) as UniffiCallbackFunction
    }
    override fun write(buf: UniffiBufferCursor, value: UniffiCallbackFunction) {
        buf.writeLong(Pointer.nativeValue(CallbackReference.getFunctionPointer(value)))
    }
}

private object UniffiFfiSerializerBoundCallback: UniffiFfiSerializer<UniffiBoundCallback> {
    override fun size(): Long = 16
    override fun read(buf: UniffiBufferCursor): UniffiBoundCallback {
        val callback = UniffiFfiSerializerCallback.read(buf)
        val data = UniffiFfiSerializerHandle.read(buf)
        return UniffiBoundCallback(callback, data)
    }
    override fun write(buf: UniffiBufferCursor, value: UniffiBoundCallback) {
        UniffiFfiSerializerCallback.write(buf, value.callback)
        UniffiFfiSerializerHandle.write(buf, value.data)
    }
}

private object UniffiFfiSerializerRustBuffer: UniffiFfiSerializer<RustBuffer.ByValue> {
    override fun size(): Long = 24
    override fun read(buf: UniffiBufferCursor): RustBuffer.ByValue {
        var rustBuffer = RustBuffer.ByValue()
        rustBuffer.capacity = buf.readLong()
        rustBuffer.len = buf.readLong()
        rustBuffer.data = Pointer(buf.readLong())
        return rustBuffer
    }
    override fun write(buf: UniffiBufferCursor, value: RustBuffer.ByValue) {
        buf.writeLong(value.capacity)
        buf.writeLong(value.len)
        buf.writeLong(Pointer.nativeValue(value.data))
    }
}

private object UniffiFfiSerializerUniffiRustCallStatus: UniffiFfiSerializer<UniffiRustCallStatus.ByValue> {
    override fun size(): Long = 32
    override fun read(buf: UniffiBufferCursor): UniffiRustCallStatus.ByValue {
        var status = UniffiRustCallStatus.ByValue()
        status.code = buf.readByte()
        status.error_buf = UniffiFfiSerializerRustBuffer.read(buf)
        return status
    }
    override fun write(buf: UniffiBufferCursor, value: UniffiRustCallStatus.ByValue) {
        buf.writeByte(value.code)
        UniffiFfiSerializerRustBuffer.write(buf, value.error_buf)
    }
}

private object UniffiFfiSerializerForeignFutureDroppedCallback: UniffiFfiSerializer<UniffiBoundCallback?> {
    override fun size(): Long = 24
    override fun read(buf: UniffiBufferCursor): UniffiBoundCallback? {
        val code = UniffiFfiSerializerByte.read(buf)
        if (code == 0.toByte()) {
            buf.advance(16);
            return null
        }
        return UniffiFfiSerializerBoundCallback.read(buf)
    }
    override fun write(buf: UniffiBufferCursor, value: UniffiBoundCallback?) {
        if (value == null) {
            UniffiFfiSerializerByte.write(buf, 0)
            buf.advance(16)
            return;
        }
        UniffiFfiSerializerByte.write(buf, 1)
        UniffiFfiSerializerBoundCallback.write(buf, value)
    }
}
