// This is a helper for safely working with byte buffers returned from the Rust code.
// A rust-owned buffer is represented by its capacity, its current length, and a
// pointer to the underlying data.

/**
 * @suppress
 */
@Structure.FieldOrder("capacity", "len", "data")
open class RustBuffer : Structure() {
    // Note: `capacity` and `len` are actually `ULong` values, but JVM only supports signed values.
    // When dealing with these fields, make sure to call `toULong()`.
    @JvmField var capacity: Long = 0
    @JvmField var len: Long = 0
    @JvmField var data: Pointer? = null

    class ByValue: RustBuffer(), Structure.ByValue
    class ByReference: RustBuffer(), Structure.ByReference

   internal fun setValue(other: RustBuffer) {
        capacity = other.capacity
        len = other.len
        data = other.data
    }

    companion object {
        internal fun alloc(size: ULong = 0UL) = uniffiRustCall() { status ->
            // Note: need to convert the size to a `Long` value to make this work with JVM.
            UniffiLib.{{ ci.ffi_rustbuffer_alloc().name() }}(size.toLong(), status)
        }.also {
            if(it.data == null) {
               throw RuntimeException("RustBuffer.alloc() returned null data pointer (size=${size})")
           }
        }

        internal fun create(capacity: ULong, len: ULong, data: Pointer?): RustBuffer.ByValue {
            var buf = RustBuffer.ByValue()
            buf.capacity = capacity.toLong()
            buf.len = len.toLong()
            buf.data = data
            return buf
        }

        internal fun free(buf: RustBuffer.ByValue) = uniffiRustCall() { status ->
            UniffiLib.{{ ci.ffi_rustbuffer_free().name() }}(buf, status)
        }
    }

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer() =
        this.data?.getByteBuffer(0, this.len)?.also {
            it.order(ByteOrder.BIG_ENDIAN)
        }
}

// This is a helper for safely passing byte references into the rust code.
// It's not actually used at the moment, because there aren't many things that you
// can take a direct pointer to in the JVM, and if we're going to copy something
// then we might as well copy it into a `RustBuffer`. But it's here for API
// completeness.

@Structure.FieldOrder("len", "data")
internal open class ForeignBytes : Structure() {
    @JvmField var len: Int = 0
    @JvmField var data: Pointer? = null

    class ByValue : ForeignBytes(), Structure.ByValue
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
// The returned `ForeignBytes.ByValue` is only valid for the duration of
// the FFI call; the Rust side treats it as a borrow.
internal object FfiConverterByRefBytes : FfiConverter<java.nio.ByteBuffer, ForeignBytes.ByValue> {
    override fun lower(value: java.nio.ByteBuffer): ForeignBytes.ByValue {
        require(value.isDirect) { "UniFFI zero-copy &[u8] requires a direct ByteBuffer. Use ByteBuffer.allocateDirect()." }
        val remaining = value.remaining()
        val fb = ForeignBytes.ByValue()
        fb.len = remaining
        // Zero-length direct buffers: skip getDirectBufferPointer (platform-variable behavior)
        // and pass null. The Rust side treats (null, 0) as &[].
        fb.data = if (remaining == 0) null else com.sun.jna.Native.getDirectBufferPointer(value)
        return fb
    }

    override fun lift(value: ForeignBytes.ByValue): java.nio.ByteBuffer =
        error("ByRef bytes cannot be lifted: zero-copy &[u8] only flows foreign->Rust")

    override fun read(buf: java.nio.ByteBuffer): java.nio.ByteBuffer =
        error("ByRef bytes cannot be read from a buffer: zero-copy &[u8] only flows foreign->Rust")

    override fun write(value: java.nio.ByteBuffer, buf: java.nio.ByteBuffer): Unit =
        error("ByRef bytes cannot be written to a buffer: zero-copy &[u8] only flows foreign->Rust")

    override fun allocationSize(value: java.nio.ByteBuffer): ULong =
        error("ByRef bytes have no RustBuffer allocation size: zero-copy &[u8] only flows foreign->Rust")
}
