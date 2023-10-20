{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.locks.ReentrantLock") }}

interface ForeignCallback : com.sun.jna.Callback {
    public fun invoke(handle: UniffiHandle, method: Int, argsData: Pointer, argsLen: Int, outBuf: RustBufferByReference): Int
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
internal const val IDX_CALLBACK_FREE = 0
// Callback return codes
internal const val UNIFFI_CALLBACK_SUCCESS = 0
internal const val UNIFFI_CALLBACK_ERROR = 1
internal const val UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

public abstract class FfiConverterCallbackInterface<CallbackInterface>: FfiConverter<CallbackInterface, UniffiHandle> {
    internal val slab = UniffiSlab<CallbackInterface>()

    override fun lift(value: UniffiHandle): CallbackInterface {
        return slab.get(value)
    }

    override fun read(buf: ByteBuffer) = lift(buf.getLong())

    override fun lower(value: CallbackInterface) = slab.insert(value)

    override fun allocationSize(value: CallbackInterface) = 8

    override fun write(value: CallbackInterface, buf: ByteBuffer) {
        buf.putLong(lower(value))
    }
}
