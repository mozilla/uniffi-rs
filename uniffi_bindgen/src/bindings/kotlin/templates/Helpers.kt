// A handful of classes and functions to support the generated data structures.
// This would be a good candidate for isolating in its own ffi-support lib.

{% if ci.iter_object_definitions().len() > 0 %}
abstract class FFIObject(
    private val handle: AtomicLong
) {
    open fun destroy() {
        this.handle.set(0L)
    }

    internal inline fun <R> callWithHandle(block: (handle: Long) -> R) =
        this.handle.get().let { handle -> 
            if (handle != 0L) {
                block(handle)
            } else {
                throw IllegalStateException("${this.javaClass.simpleName} object has already been destroyed")
            }
        }
}

inline fun <T : FFIObject, R> T.use(block: (T) -> R) =
    try {
        block(this)
    } finally {
        try {
            this.destroy()
        } catch (e: Throwable) {
            // swallow
        }
    }
{% endif %}

{% if ci.iter_callback_interface_definitions().len() > 0 %}
internal typealias Handle = Long
internal class ConcurrentHandleMap<T>(
    private val leftMap: MutableMap<Handle, T> = mutableMapOf(),
    private val rightMap: MutableMap<T, Handle> = mutableMapOf()
) {
    private val lock = java.util.concurrent.locks.ReentrantLock()
    private val currentHandle = AtomicLong(0L)
    private val stride = 1L

    fun insert(obj: T): Handle =
        lock.withLock {
            rightMap[obj] ?:
                currentHandle.getAndAdd(stride)
                    .also { handle ->
                        leftMap[handle] = obj
                        rightMap[obj] = handle
                    }
            }

    fun <R> callWithResult(handle: Handle, fn: (T) -> R): R =
        lock.withLock {
            leftMap[handle] ?: throw RuntimeException("Panic: handle not in handlemap")
        }.let { obj ->
            fn.invoke(obj)
        }

    fun get(handle: Handle) = lock.withLock {
        leftMap[handle]
    }

    fun delete(handle: Handle) { 
        this.remove(handle)
    }

    fun remove(handle: Handle): T? =
        lock.withLock {
            leftMap.remove(handle)?.let { obj ->
                rightMap.remove(obj)
                obj
            }
        }
}

interface ForeignCallback : com.sun.jna.Callback {
    public fun invoke(handle: Long, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue
}

// Magic number for the Rust proxy to call using the same mechanism as every other method,
// to free the callback once it's dropped by Rust.
internal const val IDX_CALLBACK_FREE = 0

internal abstract class CallbackInternals<CallbackInterface>(
    val foreignCallback: ForeignCallback
) {
    val handleMap = ConcurrentHandleMap<CallbackInterface>()

    // Registers the foreign callback with the Rust side.
    // This method is generated for each callback interface.
    abstract fun register(lib: _UniFFILib)

    fun drop(handle: Long): RustBuffer.ByValue {
        return handleMap.remove(handle).let { RustBuffer.ByValue() }
    }

    fun lift(n: Long) = handleMap.get(n)

    fun read(buf: ByteBuffer) = lift(buf.getLong())

    fun lower(v: CallbackInterface) =
        handleMap.insert(v).also {
            assert(handleMap.get(it) === v) { "Handle map is not returning the object we just placed there. This is a bug in the HandleMap." }
        }

    fun write(v: CallbackInterface, buf: RustBufferBuilder) =
        buf.putLong(lower(v))
}
{% endif %}