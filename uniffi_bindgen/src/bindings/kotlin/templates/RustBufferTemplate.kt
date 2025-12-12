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

@Structure.FieldOrder("len", "data")
internal open class ForeignBytes : Structure() {
    @JvmField var len: Int = 0
    @JvmField var data: Pointer? = null
}

// Converter for `&[u8]` / `[ByRef] bytes` arguments.
//
// Only `lower` is valid — zero-copy byte buffers only flow foreign -> Rust,
// and only in argument position. `lift`, `read`, `write`, and
// `allocationSize` have no sound implementation here and all panic at
// runtime. The `FfiConverter` interface is implemented so that the
// compiler enforces the full method set (rather than relying on eyeball).
//
// The provided `ByteBuffer` MUST be direct — only direct buffers have a
// stable native address that JNA can expose via `getDirectBufferPointer`.
// The returned `ForeignBytes` is only valid for the duration of
// the FFI call; the Rust side treats it as a borrow.
internal object FfiConverterByRefBytes : FfiConverter<java.nio.ByteBuffer, ForeignBytes> {
    override fun lower(value: java.nio.ByteBuffer): ForeignBytes {
        require(value.isDirect) { "UniFFI zero-copy &[u8] requires a direct ByteBuffer. Use ByteBuffer.allocateDirect()." }
        val remaining = value.remaining()
        val fb = ForeignBytes()
        fb.len = remaining
        // Zero-length direct buffers: skip getDirectBufferPointer (platform-variable behavior)
        // and pass null. The Rust side treats (null, 0) as &[].
        fb.data = if (remaining == 0) null else com.sun.jna.Native.getDirectBufferPointer(value)
        return fb
    }

    override fun lift(value: ForeignBytes): java.nio.ByteBuffer =
        error("ByRef bytes cannot be lifted: zero-copy &[u8] only flows foreign->Rust")

    override fun read(buf: java.nio.ByteBuffer): java.nio.ByteBuffer =
        error("ByRef bytes cannot be read from a buffer: zero-copy &[u8] is only supported in argument position, not nested in records/options/etc.")

    override fun write(value: java.nio.ByteBuffer, buf: java.nio.ByteBuffer): Unit =
        error("ByRef bytes cannot be written to a buffer: zero-copy &[u8] is only supported in argument position, not nested in records/options/etc.")

    override fun allocationSize(value: java.nio.ByteBuffer): ULong =
        error("ByRef bytes have no RustBuffer allocation size: zero-copy &[u8] is only supported in argument position, not nested in records/options/etc.")
}
