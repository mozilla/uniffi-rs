// This is a helper for safely working with byte buffers returned from the Rust code.
// It's basically a wrapper around a length and a data pointer, corresponding to the
// `ffi_support::ByteBuffer` struct on the rust side.
//
// It's lightly modified from the version we use in application-services.

@Structure.FieldOrder("len", "data")
open class RustBuffer : Structure() {
    @JvmField var len: Long = 0
    @JvmField var data: Pointer? = null

    class ByValue : RustBuffer(), Structure.ByValue

    companion object {
        internal fun alloc(size: Int): RustBuffer.ByValue {
            return _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_alloc().name() }}(size)
        }

        internal fun free(buf: RustBuffer.ByValue) {
            return _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_free().name() }}(buf)
        }
    }

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer(): ByteBuffer? {
        return this.data?.let {
            val buf = it.getByteBuffer(0, this.len)
            buf.order(ByteOrder.BIG_ENDIAN)
            return buf
        }
    }
}