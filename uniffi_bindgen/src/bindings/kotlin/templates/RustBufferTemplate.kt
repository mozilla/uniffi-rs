// This is a helper for safely working with byte buffers returned from the Rust code.
// A rust-owned buffer is represented by its capacity, its current length, and a
// pointer to the underlying data.

/**
 * @suppress
 */
open class RustBuffer(val capacity: Long, var len: Long, val data: Pointer) {
    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer() =
        this.data.getByteBuffer(0, this.len)?.also {
            it.order(ByteOrder.BIG_ENDIAN)
        }

    companion object {
        internal fun default() = RustBuffer(0, 0, Pointer(0))

        internal fun alloc(size: ULong = 0UL): RustBuffer {
            val ffiBuffer = Memory(24)
            val argsCursor = UniffiBufferCursor(ffiBuffer)
            UniffiFfiSerializerLong.write(argsCursor, size.toLong());
            UniffiLib.{{ ci.pointer_ffi_rustbuffer_alloc() }}(ffiBuffer);
            val returnCursor = UniffiBufferCursor(ffiBuffer)
            val allocatedBuffer = UniffiFfiSerializerRustBuffer.read(returnCursor)
            return allocatedBuffer
        }

        internal fun free(buf: RustBuffer) {
            val ffiBuffer = Memory(24)
            val argsCursor = UniffiBufferCursor(ffiBuffer)
            UniffiFfiSerializerRustBuffer.write(argsCursor, buf);
            UniffiLib.{{ ci.pointer_ffi_rustbuffer_free() }}(ffiBuffer);
        }
    }
}
