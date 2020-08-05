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
        internal fun alloc(size: Int) =
            _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_alloc().name() }}(size)

        internal fun free(buf: RustBuffer.ByValue) =
            _UniFFILib.INSTANCE.{{ ci.ffi_bytebuffer_free().name() }}(buf)
    }

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer() = 
        this.data?.getByteBuffer(0, this.len)?.also {
            it.order(ByteOrder.BIG_ENDIAN)
        }
}