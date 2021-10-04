// Interface to convert between the Kotlin type and a FFI type
interface FFIConverter<K, F> {
    fun lift(v: F): K;
    fun lower(v: K): F;
    fun read(buf: ByteBuffer): K;
    fun write(v: K, buf: RustBufferBuilder);
}

// FFIConverter that implements lift and lower() by reading/writing to a `RustBuffer`
interface FFIConverterRustBuffer<K> : FFIConverter<K, RustBuffer.ByValue> {
    override fun lift(v: RustBuffer.ByValue): K {
        val buf = v.asByteBuffer()!!
        try {
           val item = read(buf)
           if (buf.hasRemaining()) {
               throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
           }
           return item
        } finally {
            RustBuffer.free(v)
        }
    }

    override fun lower(v: K): RustBuffer.ByValue {
        // TODO: maybe we can calculate some sort of initial size hint?
        val buf = RustBufferBuilder()
        try {
            write(v, buf)
            return buf.finalize()
        } catch (e: Throwable) {
            buf.discard()
            throw e
        }
    }
}

// Wrap the output of a FFIConverter to implement FFIConverter for the new type
abstract class FFIWrapper<N, K, F>(val wrapped: FFIConverter<K, F>) {
    // Wrap the FFIConverter output into the new type N
    abstract fun wrap(v: K): N

    // Unwrap N into the FFIConverter input
    abstract fun unwrap(v: N): K

    fun lift(v: F) = wrap(wrapped.lift(v))
    fun lower(v: N) = wrapped.lower(unwrap(v))
    fun read(buf: ByteBuffer) = wrap(wrapped.read(buf))
    fun write(v: N, buf: RustBufferBuilder) = wrapped.write(unwrap(v), buf)
}

