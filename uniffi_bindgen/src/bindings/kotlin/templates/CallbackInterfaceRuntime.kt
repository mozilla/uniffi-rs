{{- self.add_import("java.util.concurrent.atomic.AtomicLong") }}
{{- self.add_import("java.util.concurrent.locks.ReentrantLock") }}
{{- self.add_import("kotlin.concurrent.withLock") }}

internal typealias Handle = Long
internal class ConcurrentHandleMap<T>(
    private val leftMap: MutableMap<Handle, T> = mutableMapOf(),
) {
    private val lock = java.util.concurrent.locks.ReentrantLock()
    private val currentHandle = AtomicLong(0L)
    private val stride = 1L

    fun insert(obj: T): Handle =
        lock.withLock {
            currentHandle.getAndAdd(stride)
                .also { handle ->
                    leftMap[handle] = obj
                }
            }

    fun get(handle: Handle) = lock.withLock {
        leftMap[handle] ?: throw InternalException("No callback in handlemap; this is a Uniffi bug")
    }

    fun delete(handle: Handle) {
        this.remove(handle)
    }

    fun remove(handle: Handle): T? =
        lock.withLock {
            leftMap.remove(handle)
        }
}

interface ForeignCallback : com.sun.jna.Callback {
    public fun invoke(handle: Handle, method: Int, argsData: Pointer, argsLen: Int, outBuf: RustBufferByReference): Int
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
internal const val IDX_CALLBACK_FREE = 0
// Callback return codes
internal const val UNIFFI_CALLBACK_SUCCESS = 0
internal const val UNIFFI_CALLBACK_ERROR = 1
internal const val UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

public abstract class FfiConverterCallbackInterface<CallbackInterface>: FfiConverter<CallbackInterface, Handle> {
    internal val handleMap = ConcurrentHandleMap<CallbackInterface>()

    internal fun drop(handle: Handle) {
        handleMap.remove(handle)
    }

    override fun lift(value: Handle): CallbackInterface {
        return handleMap.get(value)
    }

    override fun read(buf: ByteBuffer) = lift(buf.getLong())

    override fun lower(value: CallbackInterface) = handleMap.insert(value)

    override fun allocationSize(value: CallbackInterface) = 8

    override fun write(value: CallbackInterface, buf: ByteBuffer) {
        buf.putLong(lower(value))
    }
}
